// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::os::unix::io::AsRawFd;

use logger::{debug, error, warn};
use polly::event_manager::{EventManager, Subscriber};
use utils::epoll::{EpollEvent, EventSet};

use crate::report_balloon_event_fail;
use crate::virtio::{
    balloon::device::Balloon, VirtioDevice, DEFLATE_INDEX, INFLATE_INDEX, STATS_INDEX,
};

impl Balloon {
    fn process_activate_event(&self, event_manager: &mut EventManager) {
        debug!("balloon: activate event");
        if let Err(e) = self.activate_evt.read() {
            error!("Failed to consume balloon activate event: {:?}", e);
        }
        let activate_fd = self.activate_evt.as_raw_fd();
        // The subscriber must exist as we previously registered activate_evt via
        // `interest_list()`.
        let self_subscriber = match event_manager.subscriber(activate_fd) {
            Ok(subscriber) => subscriber,
            Err(e) => {
                error!("Failed to process balloon activate evt: {:?}", e);
                return;
            }
        };

        // Interest list changes when the device is activated.
        let interest_list = self.interest_list();
        for event in interest_list {
            event_manager
                .register(event.data() as i32, event, self_subscriber.clone())
                .unwrap_or_else(|e| {
                    error!("Failed to register balloon events: {:?}", e);
                });
        }

        event_manager.unregister(activate_fd).unwrap_or_else(|e| {
            error!("Failed to unregister balloon activate evt: {:?}", e);
        });
    }
}

impl Subscriber for Balloon {
    fn process(&mut self, event: &EpollEvent, evmgr: &mut EventManager) {
        let source = event.fd();
        let event_set = event.event_set();
        let supported_events = EventSet::IN;

        if !supported_events.contains(event_set) {
            warn!(
                "Received unknown event: {:?} from source: {:?}",
                event_set, source
            );
            return;
        }

        if self.is_activated() {
            let virtq_inflate_ev_fd = self.queue_evts[INFLATE_INDEX].as_raw_fd();
            let virtq_deflate_ev_fd = self.queue_evts[DEFLATE_INDEX].as_raw_fd();
            let virtq_stats_ev_fd = self.queue_evts[STATS_INDEX].as_raw_fd();
            let stats_timer_fd = self.stats_timer.as_raw_fd();
            let activate_fd = self.activate_evt.as_raw_fd();

            // Looks better than C style if/else if/else.
            match source {
                _ if source == virtq_inflate_ev_fd => self
                    .process_inflate_queue_event()
                    .unwrap_or_else(report_balloon_event_fail),
                _ if source == virtq_deflate_ev_fd => self
                    .process_deflate_queue_event()
                    .unwrap_or_else(report_balloon_event_fail),
                _ if source == virtq_stats_ev_fd => self
                    .process_stats_queue_event()
                    .unwrap_or_else(report_balloon_event_fail),
                _ if source == stats_timer_fd => self
                    .process_stats_timer_event()
                    .unwrap_or_else(report_balloon_event_fail),
                _ if activate_fd == source => self.process_activate_event(evmgr),
                _ => {
                    warn!("Balloon: Spurious event received: {:?}", source);
                }
            };
        } else {
            warn!(
                "Balloon: The device is not yet activated. Spurious event received: {:?}",
                source
            );
        }
    }

    fn interest_list(&self) -> Vec<EpollEvent> {
        // This function can be called during different points in the device lifetime:
        //  - shortly after device creation,
        //  - on device activation (is-activated already true at this point),
        //  - on device restore from snapshot.
        if self.is_activated() {
            let mut events = vec![
                EpollEvent::new(
                    EventSet::IN,
                    self.queue_evts[INFLATE_INDEX].as_raw_fd() as u64,
                ),
                EpollEvent::new(
                    EventSet::IN,
                    self.queue_evts[DEFLATE_INDEX].as_raw_fd() as u64,
                ),
            ];
            if self.stats_enabled() {
                events.extend(vec![
                    EpollEvent::new(
                        EventSet::IN,
                        self.queue_evts[STATS_INDEX].as_raw_fd() as u64,
                    ),
                    EpollEvent::new(EventSet::IN, self.stats_timer.as_raw_fd() as u64),
                ]);
            }
            events
        } else {
            vec![EpollEvent::new(
                EventSet::IN,
                self.activate_evt.as_raw_fd() as u64,
            )]
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::virtio::balloon::test_utils::set_request;
    use crate::virtio::test_utils::{default_mem, VirtQueue};
    use vm_memory::GuestAddress;

    #[test]
    fn test_event_handler() {
        let mut event_manager = EventManager::new().unwrap();
        let mut balloon = Balloon::new(0, true, 10, false).unwrap();
        let mem = default_mem();
        let infq = VirtQueue::new(GuestAddress(0), &mem, 16);
        balloon.set_queue(INFLATE_INDEX, infq.create_queue());

        let balloon = Arc::new(Mutex::new(balloon));
        event_manager.add_subscriber(balloon.clone()).unwrap();

        // Push a queue event, use the inflate queue in this test.
        {
            let addr = 0x100;
            set_request(&infq, 0, addr, 4, 0);
            balloon.lock().unwrap().queue_evts[INFLATE_INDEX]
                .write(1)
                .unwrap();
        }

        // EventManager should report no events since balloon has only registered
        // its activation event so far (even though there is also a queue event pending).
        let ev_count = event_manager.run_with_timeout(50).unwrap();
        assert_eq!(ev_count, 0);

        // Manually force a queue event and check it's ignored pre-activation.
        {
            let mut b = balloon.lock().unwrap();
            let raw_infq_evt = b.queue_evts[INFLATE_INDEX].as_raw_fd() as u64;
            // Artificially push event.
            b.process(
                &EpollEvent::new(EventSet::IN, raw_infq_evt),
                &mut event_manager,
            );
            // Validate there was no queue operation.
            assert_eq!(infq.used.idx.get(), 0);
        }

        // Now activate the device.
        balloon.lock().unwrap().activate(mem.clone()).unwrap();
        // Process the activate event.
        let ev_count = event_manager.run_with_timeout(50).unwrap();
        assert_eq!(ev_count, 1);

        // Handle the previously pushed queue event through EventManager.
        event_manager
            .run_with_timeout(100)
            .expect("Metrics event timeout or error.");
        // Make sure the data queue advanced.
        assert_eq!(infq.used.idx.get(), 1);
    }
}

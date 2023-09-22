// Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::os::unix::io::AsRawFd;

use event_manager::{EventOps, Events, MutEventSubscriber};
use logger::{error, warn};
use utils::epoll::EventSet;

use crate::devices::virtio::block::vhost_user::device::BlockVhostUser;

impl BlockVhostUser {
    fn register_activate_event(&self, ops: &mut EventOps) {
        if let Err(err) = ops.add(Events::new(&self.activate_evt, EventSet::IN)) {
            error!("Failed to register activate event: {}", err);
        }
    }

    fn process_activate_event(&self, ops: &mut EventOps) {
        if let Err(err) = self.activate_evt.read() {
            error!("Failed to consume block activate event: {:?}", err);
        }
        if let Err(err) = ops.remove(Events::new(&self.activate_evt, EventSet::IN)) {
            error!("Failed to un-register activate event: {}", err);
        }
    }

    fn is_activated(&self) -> bool {
        self.device_state.is_activated()
    }
}

impl MutEventSubscriber for BlockVhostUser {
    // Handle an event for queue or rate limiter.
    fn process(&mut self, event: Events, ops: &mut EventOps) {
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
            let activate_fd = self.activate_evt.as_raw_fd();
            // Looks better than C style if/else if/else.
            match source {
                _ if activate_fd == source => self.process_activate_event(ops),
                _ => warn!("BlockVhost: Spurious event received: {:?}", source),
            }
        } else {
            warn!(
                "BlockVhost: The device is not yet activated. Spurious event received: {:?}",
                source
            );
        }
    }

    fn init(&mut self, ops: &mut EventOps) {
        // This function can be called during different points in the device lifetime:
        //  - shortly after device creation,
        //  - on device activation (is-activated already true at this point),
        //  - on device restore from snapshot.
        if self.is_activated() {
            error!("This a vhost backed block. Not sure why I received this event");
        } else {
            self.register_activate_event(ops);
        }
    }
}

use super::super::EpollHandlerPayload;
use super::INTERRUPT_STATUS_USED_RING;

use sys_util::EventFd;
use vhost_backend::Vhost;
use DeviceEventT;
use EpollHandler;

use std::os::unix::io::{AsRawFd, RawFd};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

pub const VHOST_IRQ_AVAILABLE: DeviceEventT = 0;
pub const KILL_EVENT: DeviceEventT = 1;

pub struct VhostEpollHandler<T: Vhost> {
    vhost_dev: T,
    interrupt_status: Arc<AtomicUsize>,
    interrupt_evt: EventFd,
    queue_evt: EventFd,
}

impl<T: Vhost> VhostEpollHandler<T> {
    /// Construct a new, empty event handler for vhost-based devices.
    ///
    /// # Arguments
    /// * `vhost_dev` - the vhost-based device info
    /// * `interrupt_status` - semaphore before triggering interrupt event
    /// * `interrupt_evt` EventFd for signaling an MMIO interrupt that the guest
    ///                   driver is listening to
    /// * `queue_evt` - EventFd used by the handle to monitor queue events
    pub fn new(
        vhost_dev: T,
        interrupt_status: Arc<AtomicUsize>,
        interrupt_evt: EventFd,
        queue_evt: EventFd,
    ) -> VhostEpollHandler<T> {
        VhostEpollHandler {
            vhost_dev,
            interrupt_status,
            interrupt_evt,
            queue_evt,
        }
    }

    fn signal_used_queue(&self) {
        self.interrupt_status
            .fetch_or(INTERRUPT_STATUS_USED_RING as usize, Ordering::SeqCst);
        self.interrupt_evt.write(1).unwrap();
    }

    pub fn get_queue_evt(&self) -> RawFd {
        return self.queue_evt.as_raw_fd();
    }

    pub fn get_device(&self) -> &T {
        return &self.vhost_dev;
    }
}

impl<T: Vhost> EpollHandler for VhostEpollHandler<T>
where
    T: std::marker::Send,
{
    fn handle_event(&mut self, device_event: DeviceEventT, _: u32, _: EpollHandlerPayload) {
        let mut needs_interrupt = false;

        match device_event {
            VHOST_IRQ_AVAILABLE => {
                if let Err(e) = self.queue_evt.read() {
                    error!("failed reading queue EventFd: {:?}", e);
                    return;
                }
                needs_interrupt = true;
                // TODO dpopa@: after changing handle_event's signature to return Result, uncomment this
                //self.queue_evt.read().map_err(Error::VhostIrqRead)?;
            }
            KILL_EVENT => {
                //TODO: call API for device removal here
                info!("vhost device removed")
            }
            _ => panic!("unknown token for vhost device"),
        }

        if needs_interrupt {
            self.signal_used_queue();
        }
    }
}

pub struct VhostEpollConfig {
    queue_evt_token: u64,
    kill_token: u64,
    epoll_raw_fd: RawFd,
    sender: mpsc::Sender<Box<EpollHandler>>,
}

impl VhostEpollConfig {
    pub fn new(
        first_token: u64,
        epoll_raw_fd: RawFd,
        sender: mpsc::Sender<Box<EpollHandler>>,
    ) -> Self {
        VhostEpollConfig {
            queue_evt_token: first_token,
            kill_token: first_token + 1,
            epoll_raw_fd,
            sender,
        }
    }
    pub fn get_sender(&self) -> mpsc::Sender<Box<EpollHandler>> {
        self.sender.clone()
    }

    pub fn get_raw_epoll_fd(&self) -> RawFd {
        self.epoll_raw_fd
    }

    pub fn get_kill_token(&self) -> u64 {
        self.kill_token
    }

    pub fn get_queue_evt_token(&self) -> u64 {
        self.queue_evt_token
    }
}

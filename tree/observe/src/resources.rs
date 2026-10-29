//! Caller-owned enumeration resources, separate from names and acquired facts.

use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationBackend {
    Standard,
    NativeDirent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationConfig {
    pub read_size: usize,
    pub max_idle_buffers: usize,
    pub max_idle_capacity: usize,
    /// Refuse rather than silently ignoring explicit native-backend requirements.
    pub require_native: bool,
}
impl Default for ObservationConfig {
    fn default() -> Self {
        Self {
            read_size: 64 * 1024,
            max_idle_buffers: 16,
            max_idle_capacity: 1024 * 1024,
            require_native: false,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceError(pub &'static str);
impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for ResourceError {}

#[derive(Debug)]
struct Pool {
    config: ObservationConfig,
    idle: Vec<Vec<MaybeUninit<u8>>>,
    capacity: usize,
}
/// Cheap shared handle. Borrow never waits for an idle slot or holds a lock over I/O.
#[derive(Clone, Debug)]
pub struct ObservationResources(Arc<Mutex<Pool>>);
impl Default for ObservationResources {
    fn default() -> Self {
        Self::new(ObservationConfig::default()).expect("default resource configuration")
    }
}
impl ObservationResources {
    pub fn backend() -> ObservationBackend {
        if cfg!(all(target_os = "linux", feature = "native-dirent")) {
            ObservationBackend::NativeDirent
        } else {
            ObservationBackend::Standard
        }
    }
    fn validate(config: ObservationConfig) -> Result<(), ResourceError> {
        if config.read_size == 0 || config.read_size > 131072 {
            return Err(ResourceError("native read size must be in 1..=131072"));
        }
        if config.require_native && Self::backend() != ObservationBackend::NativeDirent {
            return Err(ResourceError("native directory backend is not active"));
        }
        Ok(())
    }
    pub fn new(config: ObservationConfig) -> Result<Self, ResourceError> {
        Self::validate(config)?;
        Ok(Self(Arc::new(Mutex::new(Pool {
            config,
            idle: Vec::new(),
            capacity: 0,
        }))))
    }
    pub fn config(&self) -> ObservationConfig {
        self.0.lock().expect("resource lock poisoned").config
    }
    /// Live leases return under these new settings. Excess idle buffers drop now.
    pub fn reconfigure(&self, config: ObservationConfig) -> Result<(), ResourceError> {
        Self::validate(config)?;
        let mut pool = self.0.lock().expect("resource lock poisoned");
        pool.config = config;
        let idle = std::mem::take(&mut pool.idle);
        pool.capacity = 0;
        for buffer in idle {
            if buffer.len() == config.read_size
                && pool.idle.len() < config.max_idle_buffers
                && buffer.capacity() <= config.max_idle_capacity.saturating_sub(pool.capacity)
            {
                pool.capacity += buffer.capacity();
                pool.idle.push(buffer);
            }
        }
        Ok(())
    }
    pub fn retained_capacity(&self) -> usize {
        self.0.lock().expect("resource lock poisoned").capacity
    }

    #[cfg_attr(not(all(target_os = "linux", feature = "native-dirent")), allow(dead_code))]
    pub(crate) fn buffer(&self) -> BufferLease {
        let (buffer, size) = {
            let mut pool = self.0.lock().expect("resource lock poisoned");
            let buffer = pool.idle.pop();
            if let Some(buffer) = &buffer {
                pool.capacity -= buffer.capacity();
            }
            (buffer, pool.config.read_size)
        };
        BufferLease {
            resources: self.clone(),
            buffer: buffer.unwrap_or_else(|| vec![MaybeUninit::uninit(); size]),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(not(all(target_os = "linux", feature = "native-dirent")), allow(dead_code))]
pub(crate) struct BufferLease {
    resources: ObservationResources,
    buffer: Vec<MaybeUninit<u8>>,
}
impl std::ops::Deref for BufferLease {
    type Target = Vec<MaybeUninit<u8>>;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}
impl std::ops::DerefMut for BufferLease {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}
impl Drop for BufferLease {
    fn drop(&mut self) {
        let mut pool = self.resources.0.lock().expect("resource lock poisoned");
        let config = pool.config;
        if self.buffer.len() == config.read_size
            && pool.idle.len() < config.max_idle_buffers
            && self.buffer.capacity() <= config.max_idle_capacity.saturating_sub(pool.capacity)
        {
            let buffer = std::mem::take(&mut self.buffer);
            pool.capacity += buffer.capacity();
            pool.idle.push(buffer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn leases_are_lazy_nonwaiting_and_return_under_current_limits() {
        let resources = ObservationResources::default();
        assert_eq!(resources.retained_capacity(), 0);
        let a = resources.buffer();
        let b = resources.buffer();
        assert_eq!(a.len(), 65536);
        drop(a);
        assert_eq!(resources.retained_capacity(), 65536);
        resources
            .reconfigure(ObservationConfig {
                max_idle_capacity: 0,
                ..ObservationConfig::default()
            })
            .unwrap();
        drop(b);
        assert_eq!(resources.retained_capacity(), 0);
        resources
            .reconfigure(ObservationConfig {
                read_size: 1024,
                max_idle_buffers: 1,
                ..ObservationConfig::default()
            })
            .unwrap();
        let a = resources.buffer();
        let b = resources.buffer();
        drop((a, b));
        assert_eq!(resources.retained_capacity(), 1024);
    }
}

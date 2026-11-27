//! SigmaOS Network Manager
//!
//! Declarative network interface and routing configuration.
//! Manages `NetInterface` records (IP/mask, state, MTU) and a routing table.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Link state of a network interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkState {
    Up,
    Down,
}

/// A network interface descriptor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetInterface {
    pub name: String,
    pub address: String,   // e.g. "192.168.1.10/24"
    pub state: LinkState,
    pub mtu: u16,
}

/// A routing table entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub destination: String,  // e.g. "0.0.0.0/0"
    pub gateway: String,      // e.g. "192.168.1.1"
    pub interface: String,
    pub metric: u32,
}

#[derive(Debug, Error, PartialEq)]
pub enum NetError {
    #[error("interface '{0}' not found")]
    InterfaceNotFound(String),
    #[error("interface '{0}' already exists")]
    DuplicateInterface(String),
    #[error("interface name must not be empty")]
    EmptyName,
    #[error("route to '{0}' not found")]
    RouteNotFound(String),
    #[error("MTU must be at least 68")]
    InvalidMtu,
}

/// The network manager.
#[derive(Debug, Default)]
pub struct NetworkManager {
    interfaces: HashMap<String, NetInterface>,
    routes: Vec<Route>,
}

impl NetworkManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_interface(&mut self, iface: NetInterface) -> Result<(), NetError> {
        if iface.name.is_empty() {
            return Err(NetError::EmptyName);
        }
        if iface.mtu < 68 {
            return Err(NetError::InvalidMtu);
        }
        if self.interfaces.contains_key(&iface.name) {
            return Err(NetError::DuplicateInterface(iface.name.clone()));
        }
        self.interfaces.insert(iface.name.clone(), iface);
        Ok(())
    }

    pub fn set_link_state(&mut self, name: &str, state: LinkState) -> Result<(), NetError> {
        let iface = self.interfaces
            .get_mut(name)
            .ok_or_else(|| NetError::InterfaceNotFound(name.to_string()))?;
        iface.state = state;
        Ok(())
    }

    pub fn add_route(&mut self, route: Route) -> Result<(), NetError> {
        if !self.interfaces.contains_key(&route.interface) {
            return Err(NetError::InterfaceNotFound(route.interface.clone()));
        }
        self.routes.push(route);
        Ok(())
    }

    pub fn remove_route(&mut self, destination: &str) -> Result<(), NetError> {
        let before = self.routes.len();
        self.routes.retain(|r| r.destination != destination);
        if self.routes.len() == before {
            return Err(NetError::RouteNotFound(destination.to_string()));
        }
        Ok(())
    }

    pub fn interface_count(&self) -> usize { self.interfaces.len() }
    pub fn route_count(&self) -> usize { self.routes.len() }

    pub fn get_interface(&self, name: &str) -> Result<&NetInterface, NetError> {
        self.interfaces.get(name).ok_or_else(|| NetError::InterfaceNotFound(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eth0() -> NetInterface {
        NetInterface {
            name: "eth0".to_string(),
            address: "10.0.0.1/24".to_string(),
            state: LinkState::Down,
            mtu: 1500,
        }
    }

    // Arrange: add eth0
    // Act: set_link_state Up
    // Assert: state is Up
    #[test]
    fn test_set_link_state_up_aaa() {
        let mut nm = NetworkManager::new();
        nm.add_interface(eth0()).unwrap();
        nm.set_link_state("eth0", LinkState::Up).unwrap();
        assert_eq!(nm.get_interface("eth0").unwrap().state, LinkState::Up);
    }

    // Arrange: duplicate interface
    // Act: add eth0 twice
    // Assert: DuplicateInterface error
    #[test]
    fn test_duplicate_interface_rejected_aaa() {
        let mut nm = NetworkManager::new();
        nm.add_interface(eth0()).unwrap();
        let err = nm.add_interface(eth0()).unwrap_err();
        assert_eq!(err, NetError::DuplicateInterface("eth0".to_string()));
    }

    // Arrange: add eth0, add default route
    // Act: remove_route("0.0.0.0/0")
    // Assert: route_count decreases
    #[test]
    fn test_add_and_remove_route_aaa() {
        let mut nm = NetworkManager::new();
        nm.add_interface(eth0()).unwrap();
        nm.add_route(Route {
            destination: "0.0.0.0/0".to_string(),
            gateway: "10.0.0.254".to_string(),
            interface: "eth0".to_string(),
            metric: 100,
        }).unwrap();
        assert_eq!(nm.route_count(), 1);
        nm.remove_route("0.0.0.0/0").unwrap();
        assert_eq!(nm.route_count(), 0);
    }

    // Arrange: add route for non-existent interface
    // Act: add_route with unknown interface
    // Assert: InterfaceNotFound error
    #[test]
    fn test_route_requires_existing_interface_aaa() {
        let mut nm = NetworkManager::new();
        let err = nm.add_route(Route {
            destination: "0.0.0.0/0".to_string(),
            gateway: "10.0.0.1".to_string(),
            interface: "wlan0".to_string(),
            metric: 50,
        }).unwrap_err();
        assert_eq!(err, NetError::InterfaceNotFound("wlan0".to_string()));
    }

    // Arrange: MTU below minimum (68)
    // Act: add_interface with mtu=10
    // Assert: InvalidMtu error
    #[test]
    fn test_invalid_mtu_rejected_aaa() {
        let mut nm = NetworkManager::new();
        let err = nm.add_interface(NetInterface {
            name: "lo".to_string(),
            address: "127.0.0.1/8".to_string(),
            state: LinkState::Up,
            mtu: 10,
        }).unwrap_err();
        assert_eq!(err, NetError::InvalidMtu);
    }

    // Arrange: set_link_state on non-existent interface
    // Act: set_link_state("wlan99", ...)
    // Assert: InterfaceNotFound error
    #[test]
    fn test_set_state_unknown_interface_aaa() {
        let mut nm = NetworkManager::new();
        let err = nm.set_link_state("wlan99", LinkState::Up).unwrap_err();
        assert_eq!(err, NetError::InterfaceNotFound("wlan99".to_string()));
    }

    // Arrange: manager with 2 interfaces
    // Act: get_interface on each
    // Assert: interface_count == 2, get_interface returns correct entry
    #[test]
    fn test_get_interface_returns_correct_entry_aaa() {
        let mut nm = NetworkManager::new();
        nm.add_interface(eth0()).unwrap();
        nm.add_interface(NetInterface {
            name: "wlan0".to_string(),
            address: "192.168.1.50/24".to_string(),
            mtu: 1500,
            state: LinkState::Down,
        }).unwrap();
        assert_eq!(nm.interface_count(), 2);
        let iface = nm.get_interface("wlan0").unwrap();
        assert_eq!(iface.mtu, 1500);
        assert_eq!(iface.state, LinkState::Down);
    }

    // Arrange: manager with eth0 + 2 routes, remove one
    // Act: route_count after remove
    // Assert: 1
    #[test]
    fn test_route_count_decrements_on_remove_aaa() {
        let mut nm = NetworkManager::new();
        nm.add_interface(eth0()).unwrap();
        nm.add_route(Route { destination: "0.0.0.0/0".to_string(), gateway: "192.168.1.1".to_string(), interface: "eth0".to_string(), metric: 100 }).unwrap();
        nm.add_route(Route { destination: "10.0.0.0/8".to_string(), gateway: "192.168.1.254".to_string(), interface: "eth0".to_string(), metric: 200 }).unwrap();
        nm.remove_route("0.0.0.0/0").unwrap();
        assert_eq!(nm.route_count(), 1);
    }
}

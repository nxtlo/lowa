#![no_std]
/// An NFC service for reading, writing and interacting with NFC cards.
extern crate alloc;
mod errors;

use core::fmt;

use alloc::{boxed::Box, collections::btree_map::BTreeMap, vec::Vec};
use errors::{ConversionError, KernelError};
use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    /// Permissions that are given to Cards.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
    pub struct Permissions: u8 {
        /// No permissions.
        const NONE = 1 << 0;
        /// Permission for regular Cards.
        const REGULAR = 1 << 1;
        /// Permission for those with IT support capabilities.
        const IT_SUPPORT = 1 << 2;
        /// Permission for those who can operate the door systems.
        const OPEN_DOORS = 1 << 3;
        /// Permission which bypasses everything. Except ...
        const ADMIN = 1 << 4;
        /// Permission which bypasses everything.
        const SUPER_ADMIN = 1 << 5;
    }
}

impl Permissions {
    /// Returns all permission excluding [`Permissions::NONE`]
    #[inline]
    const fn privileged() -> Self {
        Self::all().symmetric_difference(Self::NONE)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
enum Position {
    Manager,
    Director,
    #[default]
    Coordinator,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    id: u16,
    permissions: Permissions,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Card")
            .field("id", &self.id)
            .field("permissions", &self.permissions)
            .finish()
    }
}

impl Card {
    /// Create a new Card.
    pub const fn new(id: u16, permissions: Permissions) -> Self {
        Self { id, permissions }
    }

    /// A default Card object.
    #[inline]
    pub const fn default() -> Self {
        Self::new(0, Permissions::REGULAR)
    }

    /// An immutable reference of this Card's permissions.
    #[inline]
    pub const fn permissions(&self) -> &Permissions {
        &self.permissions
    }

    /// Check if this Card has specific permissions.
    #[inline]
    pub const fn is(&self, perms: Permissions) -> bool {
        self.permissions.contains(perms)
    }

    #[inline]
    pub fn from_bytes<'a>(bytes: &'a [u8]) -> Result<Self, ConversionError<'a>> {
        Self::try_from(bytes)
    }

    /// Convert this Card into bytes payload ready to get sent.
    #[inline]
    pub fn as_bytes(&self) -> Vec<u8> {
        // SAFETY: We're converting from self which's always a valid value.
        unsafe { serde_json::to_vec(self).unwrap_unchecked() }
    }
}

impl<'a> TryFrom<&'a [u8]> for Card {
    type Error = ConversionError<'a>;
    /// Try to convert the given bytes into [Card] object.
    #[inline]
    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        match serde_json::from_slice(value) {
            Ok(emp) => return Ok(emp),
            Err(..) => Err(ConversionError::new("Cant convert to Card", value)),
        }
    }
}

fn m(x: Result<&str, ()>) -> Result<(), ()> {
    match x {
        Ok(..) => Ok(()),
        Err(..) => loop {},
    }
}

impl TryInto<Vec<u8>> for Card {
    type Error = ConversionError<'static>;

    /// Try to convert the given card into [Vec<u8>] of bytes.
    #[inline]
    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        match serde_json::to_vec(&self) {
            Ok(bytes) => Ok(bytes),
            Err(..) => Err(ConversionError {
                message: "Invalid bytes to convert.",
                // empty slice, the conversion failed.
                bytes: &[],
            }),
        }
    }
}

/// An interface for a lower-level system that controls the NFC cards.
/// For an example this can be an PN532 NFC reader/writer.
#[allow(unused)]
trait Kernel: Send + Sync + 'static {
    fn read(&self, card: u16) -> Result<&Card, KernelError>;
    fn read_mut(&mut self, card: u16) -> Result<&mut Card, KernelError>;
    fn write(&self, card: &Card, data: &[u8]) -> Result<(), KernelError>;
    fn sense(&self);
}

/// The base system implementation that [`NfcService`] uses.
#[must_use]
#[derive(Debug, Copy, Clone)]
struct SystemBase;

type System = SystemBase;

impl SystemBase {
    #[allow(non_upper_case_globals)]
    pub const Global: SystemBase = Self;
}

#[allow(unused_variables)]
impl Kernel for SystemBase {
    fn read(&self, card: u16) -> Result<&Card, KernelError> {
        unimplemented!("Read a card from the database")
    }

    fn read_mut(&mut self, card: u16) -> Result<&mut Card, KernelError> {
        unimplemented!("Read a card from the database?")
    }

    fn write(&self, card: &Card, data: &[u8]) -> Result<(), KernelError> {
        unimplemented!("Read a card from the database")
    }

    fn sense(&self) {
        static _CARDS: [Card; 0] = [];
        while let Some(_card) = _CARDS.iter().next() {}
    }
}

/// A basic NFC service implementation.
struct NfcService<S>
where
    S: Kernel,
{
    cards: BTreeMap<u16, Card>,
    system: S,
}

impl<K> fmt::Debug for NfcService<K>
where
    K: Kernel,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Service")
            .field("cards", &self.cards.len())
            .finish()
    }
}

impl Default for NfcService<System> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl<K> NfcService<K>
where
    K: Kernel,
{
    /// Create a new basic `NfcService`.
    #[must_use]
    #[inline]
    pub const fn new() -> NfcService<SystemBase> {
        NfcService {
            system: SystemBase::Global,
            cards: BTreeMap::new(),
        }
    }

    /// Create a new NfcService with a system provider.
    #[must_use]
    #[inline]
    pub const fn new_in(system: K) -> NfcService<K> {
        Self {
            system,
            cards: BTreeMap::new(),
        }
    }

    /// Return a reference to the current kernel of this service.
    #[inline]
    pub const fn kernel<'a>(&'a self) -> &'a K {
        &self.system
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn cards(&self) -> Box<[Card]> {
        self.cards.values().copied().collect()
    }

    pub fn unbind(&mut self, card_id: &u16) -> Option<Card> {
        self.cards.remove(card_id)
    }

    pub fn put(&mut self, card: Card) {
        let _ = self.cards.insert(card.id, card);
    }

    pub fn get(&self, card_id: &u16) -> Option<&Card> {
        self.cards.get(card_id)
    }

    pub fn contains(&self, card_id: &u16) -> bool {
        self.cards.contains_key(card_id)
    }
}

fn main() {
    let mut nfc = NfcService::<System>::new();
    nfc.put(Card::default());

    let bytes = nfc.get(&0).unwrap().as_bytes();
    match Card::try_from(&bytes[..]) {
        Ok(ref card) => log::info!("{card}"),
        Err(why) => log::debug!("{} - {:?}", why.message, why.bytes),
    }

    match Vec::<u8>::try_from(&bytes[..]) {
        Ok(_bytes) => log::debug!("{:?}", _bytes),
        Err(_why) => log::debug!("{}", _why),
    }
}

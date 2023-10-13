use std::{
    error::Error,
    fmt::{Debug, Display},
};

use rustrict::CensorStr;

use fake::{faker::name::raw, Fake};
use rand::{thread_rng, Rng};

#[allow(dead_code)]
#[derive(PartialEq, Clone, Copy, Debug)]
/// Represents the class that is joined the party.
enum Class {
    Bard,
    Artist,
    Paladin,
    Wardancer,
    Scrapper,
    None,
}

impl Class {
    /// Returns true if this [Class] is a support.
    const fn is_support(&self) -> bool {
        matches!(self, Self::Artist | Self::Bard | Self::Paladin)
    }
}

#[derive(Clone, PartialEq)]
/// Represents a single party found in the party finder.
struct Party {
    name: String,
    min_lvl: i32,
    players: Vec<Class>,
    leader: Class,
    // This is a lifetime is needed for the party-finder.
}

impl Display for Party {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Party")
            .field("name", &self.name)
            .field("min_ilvl", &self.min_lvl)
            .field("leader", &self.leader)
            .field("player_count", &self.players.len())
            .finish()
    }
}

impl Debug for Party {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

#[allow(dead_code)]
impl Party {
    fn new(name: impl ToString, min_lvl: i32, leader: Class) -> Party {
        let mut players = Vec::with_capacity(8);
        players.push(leader);
        Self {
            name: name.to_string().censor(),
            min_lvl,
            players,
            leader,
        }
    }

    /// Construct an empty party.
    /// ## Example
    /// ```
    /// const EMPTY_PARTY: Party = Party::standard();
    /// assert_eq!(EMPTY_PARTY.min_ilvl, 0);
    /// ````
    const fn standard() -> Party {
        Self {
            name: String::new(),
            players: Vec::new(),
            leader: Class::None,
            min_lvl: 0,
        }
    }

    /// An immutable reference to the players in this party.
    pub const fn players(&self) -> &Vec<Class> {
        &self.players
    }

    /// Create a party with random name and random Item LVL.
    fn random() -> Self {
        let title: String = raw::Name(fake::locales::EN).fake();
        Self {
            name: title,
            min_lvl: thread_rng().gen_range(460..1600),
            players: Vec::with_capacity(8),
            leader: Class::Wardancer,
        }
    }

    /// An immutable reference to the party leader.
    /// None is returned if no leader found.
    fn leader(&self) -> Option<&Class> {
        // First player is always the leader.
        self.players.first()
    }

    fn len(&self) -> usize {
        self.players.len()
    }

    /// Add this party to the party finder.
    /// # Example
    /// ```
    /// let pf = PartyFinder::sized(1);
    /// let mut party = Party::standard();
    /// party.add_to_finder(&mut pf);
    /// ```
    fn add_to_finder<'a>(self, finder: &'a mut PartyFinder)
    where
        Self: 'a,
    {
        let _ = finder.push(self);
    }

    /// Returns True if this party has at-least one support class.
    fn has_support(&self) -> bool {
        self.players.iter().any(|class| class.is_support())
    }

    /// Whether this party needs at least a support or not.
    fn need_support(&self) -> bool {
        self.players
            .iter()
            .filter(|class| class.is_support())
            .count()
            <= 1
    }

    /// Join this party.
    #[inline]
    fn join(&mut self, class: Class) {
        self.players.push(class);
        println!("{:?} Has joined the party.", class)
    }

    #[inline]
    fn leave(&mut self, class: Class) {
        self.players.retain(|cls| &class == cls);
        println!("{:?} Has left the party.", class)
    }

    #[inline]
    fn promote_next_player(&mut self) -> Result<(), Box<dyn Error>> {
        // We swap the leader with the next player.
        // The leader is always the first index, Any other weird behavior
        // This should panic.
        if let Some(..) = self.players.get(1) {
            self.players.swap(0, 1);
            return Ok(());
        }
        Err("No other players to promote.".into())
    }
}

#[derive(Default, Clone)]
/// A growable party finder implementation.
/// # Example
/// ```
/// let leader = Class::Artist;
/// let mut finder = PartyFinder::new();
/// let p1 = Party::new("akkan reclear", 1580, leader);
/// finder.push(p1);
/// // Titles are automatically censored. Also directly add it to the finder.
/// Party::new("QUICK 1-4 BREL NO FU***", 1580, leader)
///     .add_to_finder(&mut finder);
/// // Create a party finder with limited capacity
/// let limited = PartyFinder::from(10); // Only 10 parties can be created.
/// ```
struct PartyFinder {
    parties: Vec<Party>,
}

#[allow(dead_code)]
impl PartyFinder {
    /// Construct a new empty party finder.
    const fn new() -> PartyFinder {
        Self {
            parties: Vec::new(),
        }
    }

    /// Create a new party finder object with limited parties.
    fn sized(capacity: usize) -> PartyFinder {
        Self {
            parties: Vec::with_capacity(capacity),
        }
    }

    /// Consume Self, filtering the parties based on pred and returning it as a new [PartyFinder].
    /// # Example
    /// ```
    /// let mut pf = PartyFinder::from(vec![Party::new("akkan normal fast", 0, Class::Bard)]);
    /// let need_supps = pf.filter(|party| party.need_support() && party.name.contains("akkan"));
    /// println!("{}", need_supps)
    /// ```
    fn filter<F>(self, pred: F) -> Self
    where
        F: FnMut(&Party) -> bool,
        F: 'static,
    {
        let new: Vec<Party> = self.parties.into_iter().filter(pred).collect();
        Self::from(new)
    }

    /// The size of this party-finder.
    fn size(&self) -> usize {
        self.parties.capacity()
    }

    /// Get a reference to a party by its index.
    fn get(&self, index: usize) -> &Party {
        &self.parties[index]
    }

    /// Push a new party into this party finder.
    fn push(&mut self, party: Party) -> &mut Self {
        self.parties.push(party);
        self
    }

    /// Push multiple parties in one go.
    fn push_multipart<const S: usize>(&mut self, slice: [Party; S]) {
        for part in slice {
            let _ = self.push(part);
        }
    }

    /// Returns an iterator over this party finder's references.
    fn iter(&self) -> std::slice::Iter<'_, Party> {
        self.parties.iter()
    }

    /// Returns an iterator over this party finder's mutable references.
    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Party> {
        self.parties.iter_mut()
    }
}

impl Display for PartyFinder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PartyFinder")
            .field("parties_count", &self.parties.len())
            .field("size", &self.parties.capacity())
            .finish()
    }
}

impl Debug for PartyFinder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

// Some impls to directly access the vec.
impl AsRef<Vec<Party>> for PartyFinder {
    fn as_ref(&self) -> &Vec<Party> {
        &self.parties
    }
}

impl AsMut<Vec<Party>> for PartyFinder {
    fn as_mut(&mut self) -> &mut Vec<Party> {
        &mut self.parties
    }
}

impl Into<Vec<Party>> for PartyFinder {
    fn into(self) -> Vec<Party> {
        self.parties
    }
}

impl From<Vec<Party>> for PartyFinder {
    /// Construct a party finder from a vec.
    fn from(value: Vec<Party>) -> Self {
        PartyFinder { parties: value }
    }
}

impl<const S: usize> From<[Party; S]> for PartyFinder {
    fn from(value: [Party; S]) -> Self {
        let mut new = Self::sized(S);
        for party in value {
            new.push(party);
        }
        new
    }
}

impl IntoIterator for PartyFinder {
    type IntoIter = std::vec::IntoIter<Party>;
    type Item = Party;

    fn into_iter(self) -> Self::IntoIter {
        self.parties.into_iter()
    }
}

impl From<usize> for PartyFinder {
    fn from(value: usize) -> Self {
        Self::sized(value)
    }
}

#[cfg(unused)]
const GLOBAL_PF: PartyFinder = PartyFinder::new();
#[cfg(unused)]
const EMPTY_PARTY: Party = Party::standard();

fn main() {
    let p1 = Party::random();
    let p2 = Party::new("akkan x10 reclear.", 1600, Class::Bard);
    let mut party_finder = PartyFinder::from(5);

    {
        party_finder.push_multipart([p1, p2]);
    }
    assert_eq!(party_finder.iter().any(|party| party.has_support()), true);

    let filtered = party_finder.filter(|party| party.name.contains("x10"));
    for party in filtered {
        println!("{}", party);
    }
}

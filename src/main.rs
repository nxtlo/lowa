use std::error::Error;

use rustrict::CensorStr;

use fake::{faker::name::raw, Fake};
use rand::{seq::SliceRandom, thread_rng, Rng};

#[derive(PartialEq, Clone, Copy, Debug)]
/// Represents the class that is joined the party.
enum Class {
    Bard,
    Artist,
    Paladin,
    Wardancer,
    Scrapper,
}

impl Class {
    /// Returns true if this [Class] is a support.
    const fn is_support(&self) -> bool {
        matches!(self, Self::Artist | Self::Bard | Self::Paladin)
    }
}

#[derive(Clone)]
/// Represents a single party found in the party finder.
struct Party {
    name: String,
    min_lvl: i32,
    players: Vec<Class>,
    leader: Class,
    // This is a lifetime is needed for the party-finder.
}

impl std::fmt::Display for Party {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Party")
            .field("name", &self.name)
            .field("min_ilvl", &self.min_lvl)
            .field("leader", &self.leader)
            .field("player_count", &self.players.len())
            .finish()
    }
}

impl std::fmt::Debug for Party {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

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
    /// A read-only reference to the players in this party.
    const fn players(&self) -> &Vec<Class> {
        &self.players
    }

    /// A read-only reference to the party leader.
    /// None is returned if no leader found.
    fn leader(&self) -> Option<&Class> {
        // First player is always the leader.
        self.players.first()
    }

    fn len(&self) -> usize {
        self.players.len()
    }

    /// Add this party to the party finder.
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
    fn join(&mut self, class: &Class) {
        self.players.push(*class);
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
        let element = if let Some(class) = self.players.get(1) {
            println!("Player {:?} has been promoted to leader.", class);
            true
        } else {
            false
        };
        if element {
            self.players.swap(0, 1);
            return Ok(());
        }
        Err("No other players to promote.".into())
    }
}

#[derive(Default, Debug, Clone)]
/// A growable party finder implementation.
/// # Example
/// ```
/// let leader = Class::Artist;
/// let mut finder = PartyFinder::sized(2);
/// let p1 = Party::new("akkan reclear", 1580, leader);
/// finder.push(p1);
/// // Titles are automatically censored.
/// let p2 Party::new("QUICK 1-4 BREL NO FU***", 1580, leader);
/// finder.push(p1)
/// ```
struct PartyFinder {
    parties: Vec<Party>,
}

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

    // Consume Self, filtering the parties based on pred and returning it.
    fn filter<F: FnMut(&Party) -> bool>(self, pred: F) -> Self {
        let new: Vec<Party> = self.parties.into_iter().filter(pred).collect();
        Self::from(new)
    }

    /// Push a new party into this party finder.
    fn push(&mut self, party: Party) -> &mut Self {
        self.parties.push(party);
        self
    }

    /// Returns an iterator over this party finder's references.
    fn iter(&self) -> std::slice::Iter<'_, Party> {
        self.parties.iter()
    }
}

impl IntoIterator for PartyFinder {
    type IntoIter = std::vec::IntoIter<Party>;
    type Item = Party;

    fn into_iter(self) -> Self::IntoIter {
        self.parties.into_iter()
    }
}

impl From<Vec<Party>> for PartyFinder {
    /// Construct a party finder from a vec.
    fn from(value: Vec<Party>) -> Self {
        PartyFinder { parties: value }
    }
}

#[cfg(unused)]
const GLOBAL_PF: PartyFinder = PartyFinder::new();

fn main() {
    let mut pf = PartyFinder::sized(10);
    for _ in 0..5 {
        Party::random().add_to_finder(&mut pf);
    }

    for mut party in pf.filter(|party| party.need_support()) {
        let sup = [Class::Artist, Class::Bard, Class::Paladin]
            .choose(&mut thread_rng())
            .unwrap();

        let scrapper = Class::Scrapper;
        party.join(sup);
        party.join(&scrapper);
        party.leave(scrapper);
    }
}

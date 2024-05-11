#![no_std]
#![no_main]

use core::{
    mem::{self, ManuallyDrop},
    ptr,
};

#[derive(Debug, Default)]
pub struct Heart {
    state: bool,
}

impl Heart {
    pub const fn alive() -> Self {
        Self { state: true }
    }

    pub const fn dead() -> Self {
        Self { state: false }
    }

    #[inline]
    pub const fn is_alive(&self) -> bool {
        !self.is_dead()
    }

    #[inline]
    pub const fn is_dead(&self) -> bool {
        self.state == false
    }

    #[inline]
    #[cold]
    pub fn revive(&mut self) -> bool {
        if !self.state {
            self.state = true;
        }
        self.state
    }
}

#[derive(Debug, Default)]
pub struct Person {
    heart: ManuallyDrop<Heart>,
}

impl Person {
    #[inline]
    pub const fn new(heart: Heart) -> Self {
        Self {
            heart: ManuallyDrop::new(heart),
        }
    }

    #[inline]
    pub const fn dead() -> Self {
        Self::new(Heart::dead())
    }

    #[inline]
    pub const fn alive() -> Self {
        Self::new(Heart::alive())
    }

    #[inline]
    pub fn crash(&mut self) -> bool {
        let state = mem::replace(&mut self.heart.state, rand::random());

        if self.heart.is_dead() {
            let death = {
                let gone = mem::take(self);
                let state = gone.heart.state;
                mem::forget(gone);
                state
            };
            return death;
        }
        state
    }

    #[inline]
    pub fn transfer(mut self, mut other: Person) -> Result<Person, ()> {
        if self.heart.is_dead() {
            log::error!("you're already dead.");
            return Err(());
        }

        if other.heart.is_alive() {
            log::error!("other already alive.");
            return Err(());
        }
        unsafe { ptr::swap_nonoverlapping(&mut self.heart, &mut other.heart, 1) }
        // Drop self.
        let _ = self.kill();
        Ok(other)
    }

    #[inline]
    pub const fn kill(self) -> Heart {
        ManuallyDrop::into_inner(self.heart)
    }
}

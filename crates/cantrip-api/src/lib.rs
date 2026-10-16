//! The public surface card code is written against (docs/engine-design.md, "Layers: core, shared mechanics,
//! cards"). Card crates and `cantrip-mechanics` depend on this crate alone, so the compiler keeps them out of the
//! rules' internals: they read the state, and change it only through [`Game`].

pub use cantrip_state::{
    AbilityRef, Action, ActionKind, ActionOption, Answer, Attacker, Block, CardId, CardType, Color, ColorSet, Copiable,
    CounterKind, DayNight, Duration, EffectKind, GameState, Keyword, Keywords, Label, Mana, ManaCost, ManaPool, Marks, Obj, ObjectId,
    Outcome, PlayerId, Purpose, Request, RequestKind, StackItem, StackKind, Step, Subtype, SubtypeSet, Supertypes,
    Target, TypeSet, Word, Words, Zone, ZoneKind, ZoneReason, DelayedWhen, ExileLink, Expiry, PlayCondition, PlayGrant, PlayPermission, LastKnown, ManaRestriction, Destination, DestinationKind, DamageDivision, WAY_ADVENTURE, WAY_BACK_FACE, WAY_GRANTED, WAY_BESTOW, WAY_OVERLOAD, WAY_SPLIT_LEFT, WAY_SPLIT_RIGHT, WAY_TRANSFORMED,
};

pub use cantrip_state::{DOOR_LEFT, DOOR_RIGHT, PAY_CONVOKE, PAY_DELVE, PAY_IMPROVISE, PAY_WATERBEND, PLAYER_MANA_ABILITY};

mod ability;
mod card;
mod chars;
mod event;
mod game;
pub mod history;
pub mod intrinsic;
mod replace;

pub use ability::*;
pub use card::*;
pub use chars::*;
pub use event::*;
pub use game::*;
pub use history::*;
pub use replace::*;

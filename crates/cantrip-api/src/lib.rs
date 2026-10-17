//! The public surface card code is written against (docs/engine-design.md, "Layers: core, shared mechanics,
//! cards"). Card crates and `cantrip-mechanics` depend on this crate alone, so the compiler keeps them out of the
//! rules' internals: they read the state, and change it only through [`Game`].

pub use cantrip_state::{
    AbilityRef, Action, ActionKind, ActionOption, Answer, Attacker, Block, CardId, CardType, Color, ColorSet, Copiable, HexproofFrom,
    CounterKind, DayNight, Duration, EffectKind, Gift, GameState, Keyword, Keywords, Label, Mana, ManaCost, ManaPool, Marks, MiracleOf, NameKind, Obj, ObjectId,
    ColorUse, Outcome, PlayerId, Purpose, Question, Request, RequestKind, StackItem, StackKind, Step, Subtype, SubtypeSet, Supertypes,
    Target, TypeSet, Word, Words, Zone, ZoneKind, ZoneKinds, ZoneReason, DelayedWhen, ExileLink, Expiry, PlayCondition, PlayGrant, PlayPermission, LastKnown, ManaPayment, ManaRestriction, ManaValueSpells, SubtypeSpells, MANA_GIVES_HASTE, MANA_MAKES_UNCOUNTERABLE, Destination, DestinationKind, TotalAtMost, DamageDivision, DamageAbout, AttackCost, SearchDestination, WAY_ADVENTURE, WAY_BACK_FACE, WAY_GRANTED, WAY_GRANTED_SLOTS, WAY_GRANTED_STRIDE, GRANTED_FACE_WHOLE, GRANTED_FACE_LEFT, GRANTED_FACE_RIGHT, GRANTED_FACE_ADVENTURE, GRANTED_FACE_BACK, GRANTED_FACE_OMEN, WAY_OMEN, granted_way, granted_slot, granted_face_way, WAY_BESTOW, WAY_OVERLOAD, WAY_SPLIT_LEFT, WAY_SPLIT_RIGHT, WAY_TRANSFORMED,
};

pub use cantrip_state::{DOOR_LEFT, DOOR_RIGHT, PAY_CONVOKE, PAY_DELVE, PAY_IMPROVISE, PAY_WATERBEND, PLAYER_MANA_ABILITY, MAX_ABILITIES};

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

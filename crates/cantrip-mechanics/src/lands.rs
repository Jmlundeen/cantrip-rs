//! Land templates: the lands of a printed cycle differ in their colors and little else (CR 305). A land's file names its
//! colors and uses these. The colors are numbered as `Color` and `Mana` number them (`W`..`G` below), the basic land
//! types as `Subtype`'s discriminant (`sub(Subtype::Plains)`), because a function pointer cannot capture them.
//!
//! The "enters tapped" part of a land is `mana::enters_tapped`, `mana::enters_tapped_unless_pay_two_life` (shock lands) or
//! `enters_tapped_unless_you_control` below; the card sets it in `enters`.

use crate::library::{search_to_battlefield, Find};
use crate::{choose, effects, mana, triggers};
use cantrip_api::*;

/// White, blue, black, red and green as the numbers the generic land templates take.
pub const W: u8 = 0;
pub const U: u8 = 1;
pub const B: u8 = 2;
pub const R: u8 = 3;
pub const G: u8 = 4;

/// A subtype as the number the generic land templates take: `sub(Subtype::Island)`.
pub const fn sub(subtype: Subtype) -> u16 {
    subtype as u16
}

fn has_subtype_numbered(chars: &Chars, n: u16) -> bool {
    chars.subtypes.iter().any(|s| s as u16 == n)
}

fn controls_land_type<const A: u16>(q: &dyn Query, controller: PlayerId) -> bool {
    q.state().battlefield().iter().any(|&id| q.controller(id) == controller && has_subtype_numbered(&q.chars(id), A))
}

/// "This land enters tapped unless you control a [land type]" (CR 614.1c): read on the battlefield as it is while the land
/// enters, so the land itself is not counted, through the layers (a land of any kind may have the type). Arena of Glory
/// (Mountain), Castle Locthwain (Swamp) and Shifting Woodland (Forest) print it.
pub fn enters_tapped_unless_you_control<const TYPE: u16>(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    if !controls_land_type::<TYPE>(g, entering.controller) {
        entering.tapped = true;
    }
    Ok(())
}

fn add_two<const A: u8, const B2: u8>(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::ALL[A as usize], 1);
    g.add_mana(asker.controller, Mana::ALL[B2 as usize], 1);
    Ok(())
}

fn return_a_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    let lands: Vec<ObjectId> = g.permanents_of(player).into_iter().filter(|&id| g.chars(id).is(CardType::Land)).collect();
    if let Some(land) = choose::card(g, player, item.object, Purpose::ReturnToHand, lands)? {
        g.return_to_hand(&[land]);
    }
    Ok(())
}

/// The bounce lands' (karoos') two abilities, mana ability first: "{T}: Add {A}{B}." (`text` is its full text) and "When
/// this land enters, return a land you control to its owner's hand" (CR 603.6a): chosen as the ability resolves, not
/// targeted, and the land itself may be the one; asked whenever there is a land to choose. With `enters:
/// Some(mana::enters_tapped)` it is Azorius Chancery, Orzhov Basilica and the rest of the cycle.
pub const fn bounce_land<const A: u8, const B2: u8>(text: &'static str) -> [AbilityDef; 2] {
    let mut adds = [0u8; 6];
    adds[A as usize] += 1;
    adds[B2 as usize] += 1;
    [
        AbilityDef::Activated(ActivatedDef {
            text,
            cost: Cost { tap: true, ..Cost::NONE },
            mana: Some(add_two::<A, B2>),
            adds: ManaPool(adds),
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this land enters, return a land you control to its owner's hand.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            effect: return_a_land,
            ..TriggeredDef::BLANK
        }),
    ]
}

fn fetch_two_types<const A: u16, const B2: u16>(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_battlefield(g, item.controller, item.object, Find::ONE, false, |q, c| {
        let chars = q.chars(c);
        has_subtype_numbered(&chars, A) || has_subtype_numbered(&chars, B2)
    })?;
    Ok(())
}

/// A fetch land's ability (CR 701.23): "{T}, Pay 1 life, Sacrifice this land: Search your library for a [type A] or [type B]
/// card, put it onto the battlefield, then shuffle." `text` is the full text. Flooded Strand and Arid Mesa print it.
pub const fn fetch_land<const A: u16, const B2: u16>(text: &'static str) -> AbilityDef {
    AbilityDef::Activated(ActivatedDef {
        text,
        cost: Cost { tap: true, sacrifice_self: true, life: 1, ..Cost::NONE },
        effect: fetch_two_types::<A, B2>,
        ..ActivatedDef::BLANK
    })
}

fn fetch_basic(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_battlefield(g, item.controller, item.object, Find::ONE, false, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })?;
    Ok(())
}

/// A fetch land that finds any basic land: "{T}, Pay 1 life, Sacrifice this land: Search your library for a basic land
/// card, put it onto the battlefield, then shuffle." Prismatic Vista prints it, and so does every land that borrows its
/// text.
pub const fn fetch_basic_land(text: &'static str) -> AbilityDef {
    AbilityDef::Activated(ActivatedDef {
        text,
        cost: Cost { tap: true, sacrifice_self: true, life: 1, ..Cost::NONE },
        effect: fetch_basic,
        ..ActivatedDef::BLANK
    })
}

/// The triomes' "Cycling {3}" (CR 702.29): with `enters: Some(mana::enters_tapped)` and its three basic land types, a
/// triome. Raffine's Tower and Savai Triome print it.
pub const TRIOME_CYCLING: AbilityDef = triggers::cycling(mana(3, 0, 0, 0, 0, 0), "Cycling {3} ({3}, Discard this card: Draw a card.)");

/// The surveil lands' "When this land enters, surveil 1" (CR 701.25): with `enters: Some(mana::enters_tapped)`, Meticulous
/// Archive and Undercity Sewers.
pub const SURVEIL_LAND: AbilityDef = AbilityDef::Triggered(TriggeredDef {
    text: "When this land enters, surveil 1.",
    listens: EventMask::ZONE_CHANGE,
    matches: triggers::this_enters,
    effect: effects::surveil::<1>,
    ..TriggeredDef::BLANK
});

/// A horizon land's mana ability: "{T}, Pay 1 life: Add {A} or {B}." (`text` is its full text), the color named as it is
/// activated (CR 106.5). Horizon Canopy and Fiery Islet print it.
pub const fn pay_life_for_either(a: Color, b: Color, text: &'static str) -> AbilityDef {
    AbilityDef::Activated(ActivatedDef {
        text,
        cost: Cost { tap: true, life: 1, ..Cost::NONE },
        mana: Some(mana::add_chosen),
        any_color: true,
        colors: ColorSet(ColorSet::of(a).0 | ColorSet::of(b).0),
        adds_any: 1,
        ..ActivatedDef::BLANK
    })
}

/// A horizon land's other ability: "{1}, {T}, Sacrifice this land: Draw a card." (`text` is its full text, with the land's
/// name). Horizon Canopy and Fiery Islet print it.
pub const fn sacrifice_to_draw(text: &'static str) -> AbilityDef {
    AbilityDef::Activated(ActivatedDef {
        text,
        cost: Cost { mana: mana(1, 0, 0, 0, 0, 0), tap: true, sacrifice_self: true, ..Cost::NONE },
        effect: effects::draw::<1>,
        ..ActivatedDef::BLANK
    })
}

fn add_one<const N: u8>(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::ALL[N as usize], 1);
    Ok(())
}

fn controls_either<const A: u16, const B2: u16>(q: &dyn Query, asker: &Asker) -> bool {
    controls_land_type::<A>(q, asker.controller) || controls_land_type::<B2>(q, asker.controller)
}

/// A verge's two mana abilities (CR 602.5b): "{T}: Add {PRIMARY}." and "{T}: Add {SECOND}. Activate only if you control a
/// [type A] or a [type B]", read through the layers. Blazemire Verge and Thornspire Verge print it.
pub const fn verge<const PRIMARY: u8, const SECOND: u8, const A: u16, const B2: u16>(
    primary_text: &'static str,
    second_text: &'static str,
) -> [AbilityDef; 2] {
    let mut second = [0u8; 6];
    second[SECOND as usize] = 1;
    [
        mana::tap_for(add_one::<PRIMARY>, Mana::ALL[PRIMARY as usize], primary_text),
        AbilityDef::Activated(ActivatedDef {
            text: second_text,
            cost: Cost { tap: true, ..Cost::NONE },
            mana: Some(add_one::<SECOND>),
            adds: ManaPool(second),
            condition: Some(controls_either::<A, B2>),
            ..ActivatedDef::BLANK
        }),
    ]
}

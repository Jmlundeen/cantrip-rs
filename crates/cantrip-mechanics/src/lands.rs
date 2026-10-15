//! Land templates: the lands of a printed cycle differ in their colors and little else (CR 305). A land's file names its
//! colors and uses these. The colors are numbered as `Color` and `Mana` number them (`W`..`G` below), the basic land
//! types as `Subtype`'s discriminant (`sub(Subtype::Plains)`), because a function pointer cannot capture them.
//!
//! The "enters tapped" part of a land is `mana::enters_tapped`, `mana::enters_tapped_unless_pay_two_life` (shock lands) or
//! `enters_tapped_unless_you_control` or `enters_tapped_unless_you_control_either` below; the card sets it in `enters`.

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

/// The subtype a template's number names, as `source`'s text has that word now (CR 612.2).
fn word(q: &dyn Query, source: ObjectId, n: u16) -> Subtype {
    let printed = Subtype::ALL.into_iter().find(|&s| s as u16 == n).expect("a subtype's number");
    q.words(source).subtype(printed)
}

/// Whether `controller` controls a permanent with the land type `land_type`, through the layers (CR 305.6, 613). Arena of
/// Glory, Castle Locthwain and Shifting Woodland's "unless you control a [land type]" and Cori Mountain Monastery's "unless
/// you control a Plains or an Island" ask it.
pub fn controls_land_type(q: &dyn Query, controller: PlayerId, land_type: Subtype) -> bool {
    q.state().battlefield().iter().any(|&id| q.controller(id) == controller && q.chars(id).has_subtype(land_type))
}

/// "This land enters tapped unless you control a [land type]" (CR 614.1c): read on the battlefield as it is while the land
/// enters, so the land itself is not counted, through the layers (a land of any kind may have the type). Arena of Glory
/// (Mountain), Castle Locthwain (Swamp) and Shifting Woodland (Forest) print it.
pub fn enters_tapped_unless_you_control<const TYPE: u16>(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let land_type = word(g, entering.object, TYPE);
    if !controls_land_type(g, entering.controller, land_type) {
        entering.tapped = true;
    }
    Ok(())
}

/// "This land enters tapped unless you control a [land type] or a [land type]" (CR 614.1c): as `enters_tapped_unless_you_control`
/// with two land types, each read through the layers and the land's words (CR 612.2). Cori Mountain Monastery (Plains or
/// Island) and Mistrise Village (Mountain or Forest) print it.
pub fn enters_tapped_unless_you_control_either<const A: u16, const B: u16>(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let (a, b) = (word(g, entering.object, A), word(g, entering.object, B));
    if !controls_land_type(g, entering.controller, a) && !controls_land_type(g, entering.controller, b) {
        entering.tapped = true;
    }
    Ok(())
}

/// "Destroy target [permanent]. Its controller may search their library for a basic land card, put it onto the battlefield
/// [tapped], then shuffle": the permanent `item` targeted in `slot`, if it is still a legal target (CR 608.2b). Its controller
/// is the one it last had (CR 608.2h), asked to search after the destruction and even when the permanent was not destroyed
/// (indestructible, regenerated); declining skips the search and the shuffle. Erode (tapped), Ghost Quarter (untapped) and
/// Avengers Disassembled (tapped) print it.
pub fn destroy_target_its_controller_may_fetch_basic(g: &mut dyn Game, item: &StackItem, slot: usize, tapped: bool) -> Result<(), Suspend> {
    let Some(permanent) = crate::targets::object(g, item, slot) else { return Ok(()) };
    let owner = g.controller(permanent);
    g.destroy(permanent);
    if !effects::ask_may(g, owner, item.object, Purpose::Optional)? {
        return Ok(());
    }
    search_to_battlefield(g, owner, item.object, Find::ONE, tapped, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })?;
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
    let (a, b) = (word(g, item.object, A), word(g, item.object, B2));
    search_to_battlefield(g, item.controller, item.object, Find::ONE, false, move |q, c| {
        let chars = q.chars(c);
        chars.has_subtype(a) || chars.has_subtype(b)
    })?;
    Ok(())
}

/// A fetch land's ability (CR 701.23): "{T}, Pay 1 life, Sacrifice this land: Search your library for a [type A] or [type B]
/// card, put it onto the battlefield, then shuffle." `text` is the full text. The types are the ones the land's text named
/// as it was sacrificed (CR 612.2, 113.7a). Flooded Strand and Arid Mesa print it.
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
    controls_land_type(q, asker.controller, word(q, asker.source, A)) || controls_land_type(q, asker.controller, word(q, asker.source, B2))
}

/// A verge's two mana abilities (CR 602.5b): "{T}: Add {PRIMARY}." and "{T}: Add {SECOND}. Activate only if you control a
/// [type A] or a [type B]", read through the layers, the types as the verge's text names them (CR 612.2). Blazemire Verge and Thornspire Verge print it.
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

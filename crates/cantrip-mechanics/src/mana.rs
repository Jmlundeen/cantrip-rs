//! Mana abilities' effects that many cards print.

use cantrip_api::*;

fn add(g: &mut dyn Game, asker: &Asker, mana: Mana) -> Result<(), Suspend> {
    g.add_mana(asker.controller, mana, 1);
    Ok(())
}

pub fn add_w(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::W)
}
pub fn add_u(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::U)
}
pub fn add_b(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::B)
}
pub fn add_r(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::R)
}
pub fn add_g(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::G)
}
pub fn add_c(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    add(g, asker, Mana::C)
}
pub fn add_cc(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, Mana::C, 2);
    Ok(())
}

/// "{T}: Add {X}." as a mana ability.
pub const fn tap_for(effect: ManaEffect, mana: Mana, text: &'static str) -> AbilityDef {
    tap_for_n(effect, mana, 1, text)
}

/// "{T}: Add {X}{X}..." as a mana ability.
pub const fn tap_for_n(effect: ManaEffect, mana: Mana, amount: u8, text: &'static str) -> AbilityDef {
    let mut adds = [0u8; 6];
    adds[mana as usize] = amount;
    AbilityDef::Activated(ActivatedDef {
        text,
        cost: Cost { tap: true, ..Cost::NONE },
        mana: Some(effect),
        adds: ManaPool(adds),
        ..ActivatedDef::BLANK
    })
}

fn add_one_of_any_color(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    g.add_mana(asker.controller, color.unwrap_or(Color::White).mana(), 1);
    Ok(())
}

/// "{T}: Add one mana of any color." as a mana ability (CR 106.5). Birds of Paradise and Sylvan Caryatid print it
/// exactly so.
pub const TAP_FOR_ANY_COLOR: AbilityDef = AbilityDef::Activated(ActivatedDef {
    text: "{T}: Add one mana of any color.",
    cost: Cost { tap: true, ..Cost::NONE },
    mana: Some(add_one_of_any_color),
    any_color: true,
    adds_any: 1,
    ..ActivatedDef::BLANK
});

/// "This land enters tapped" (CR 614.1c): the modal double-faced lands' (Bala Ged Sanctuary, Haven of the Harvest).
pub fn enters_tapped(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.tapped = true;
    Ok(())
}

/// "As this land enters, you may pay 2 life. If you don't, it enters tapped" (CR 614.12a): the shocklands'. Only a
/// player with the life to pay is asked. Blood Crypt and Breeding Pool print it.
pub fn enters_tapped_unless_pay_two_life(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let paid = g.state().player(entering.controller).life >= 2
        && g.ask(entering.controller, RequestKind::May { source: entering.object, purpose: Purpose::Optional })? == Answer::Yes;
    if paid {
        g.lose_life(entering.controller, 2);
    } else {
        entering.tapped = true;
    }
    Ok(())
}

/// "As this land enters, you may pay 3 life. If you don't, it enters tapped" (CR 614.12a): the modal double-faced
/// pain lands'. Soporific Springs and Shatterskull, the Hammer Pass print it.
pub fn enters_tapped_unless_pay_three_life(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let paid = g.state().player(entering.controller).life >= 3
        && g.ask(entering.controller, RequestKind::May { source: entering.object, purpose: Purpose::Optional })? == Answer::Yes;
    if paid {
        g.lose_life(entering.controller, 3);
    } else {
        entering.tapped = true;
    }
    Ok(())
}

/// A basic land's intrinsic mana ability (CR 305.6).
pub const PLAINS: AbilityDef = tap_for(add_w, Mana::W, "{T}: Add {W}.");
pub const ISLAND: AbilityDef = tap_for(add_u, Mana::U, "{T}: Add {U}.");
pub const SWAMP: AbilityDef = tap_for(add_b, Mana::B, "{T}: Add {B}.");
pub const MOUNTAIN: AbilityDef = tap_for(add_r, Mana::R, "{T}: Add {R}.");
pub const FOREST: AbilityDef = tap_for(add_g, Mana::G, "{T}: Add {G}.");

/// Adds one mana of the color chosen as the ability was activated.
pub fn add_chosen(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    if let Some(c) = color {
        g.add_mana(asker.controller, c.mana(), 1);
    }
    Ok(())
}

/// "{T}: Add {A} or {B}." as a mana ability whose color is chosen as it is activated (CR 106.5). The fast lands,
/// Blackcleave Cliffs and Blooming Marsh among them, print it.
pub const fn tap_for_either(a: Color, b: Color, text: &'static str) -> AbilityDef {
    AbilityDef::Activated(ActivatedDef {
        text,
        cost: Cost { tap: true, ..Cost::NONE },
        mana: Some(add_chosen),
        any_color: true,
        colors: ColorSet(ColorSet::of(a).0 | ColorSet::of(b).0),
        adds_any: 1,
        ..ActivatedDef::BLANK
    })
}

/// Adds one mana of the chosen color, then the source deals 1 damage to its controller (a mana ability: no stack).
fn add_chosen_and_hurt(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    add_chosen(g, asker, color)?;
    g.damage(asker.source, Target::Player(asker.controller), 1);
    Ok(())
}

/// "{T}: Add {A} or {B}. This artifact deals 1 damage to you." The Talismans print it, every one of the five
/// enemy-color pairs.
pub const fn tap_for_either_and_hurt(a: Color, b: Color, text: &'static str) -> AbilityDef {
    AbilityDef::Activated(ActivatedDef {
        text,
        cost: Cost { tap: true, ..Cost::NONE },
        mana: Some(add_chosen_and_hurt),
        any_color: true,
        colors: ColorSet(ColorSet::of(a).0 | ColorSet::of(b).0),
        adds_any: 1,
        ..ActivatedDef::BLANK
    })
}

/// "This land enters tapped unless you control two or fewer other lands" (CR 614.1c): read on the battlefield as it is
/// while the land enters, so the land itself is not counted. Concealed Courtyard, Copperline Gorge and Darkslick
/// Shores print it.
pub fn enters_tapped_unless_two_or_fewer_other_lands(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let others = g
        .permanents_of(entering.controller)
        .into_iter()
        .filter(|&id| g.obj(id).became.is_none() && g.chars(id).is(CardType::Land))
        .count();
    if others > 2 {
        entering.tapped = true;
    }
    Ok(())
}

/// "This land enters tapped unless you control two or more other lands" (CR 614.1c): read on the battlefield as it is
/// while the land enters, so the land itself is not counted. The slow lands: Shattered Sanctum, Stormcarved Coast and
/// Sundown Pass print it.
pub fn enters_tapped_unless_two_or_more_other_lands(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let others = g
        .permanents_of(entering.controller)
        .into_iter()
        .filter(|&id| g.obj(id).became.is_none() && g.chars(id).is(CardType::Land))
        .count();
    if others < 2 {
        entering.tapped = true;
    }
    Ok(())
}

/// Sunburst (CR 702.44a): it enters with a charge counter on it for each color of mana spent to cast it, colorless
/// not being a color (CR 105.1). An object never cast spent none (CR 400.7). Engineered Explosives and Pentad Prism
/// print it.
pub fn sunburst(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let spent = g.obj(entering.object).spent;
    let colors = [Mana::W, Mana::U, Mana::B, Mana::R, Mana::G].iter().filter(|&&m| spent.get(m) > 0).count() as i16;
    entering.counters[CounterKind::Charge as usize] += colors;
    Ok(())
}

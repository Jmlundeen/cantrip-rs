//! What lands and mana sources print beyond their mana abilities, which a card writes as
//! `mana_ability(TAP, add(Mana::G), "{T}: Add {G}.")` (`cantrip_api::ability`).

use cantrip_api::*;

/// "Add {U} for each artifact you control" (CR 106.5): counted as the ability resolves, possibly none. A mana ability's
/// `add_by`: Tolarian Academy, and Uthros, Titanic Godcore's 12+ ability.
pub fn add_blue_for_each_artifact(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let artifacts = crate::artifacts_you_control(g, asker.controller);
    if artifacts > 0 {
        g.add_mana(asker.controller, Mana::U, artifacts.min(255) as u8);
    }
    Ok(())
}

/// "Add {G} for each creature you control" (CR 106.5): counted as the ability resolves, possibly none. A mana ability's
/// `add_by`: Gaea's Cradle, and Evendo, Waking Haven's 12+ ability.
pub fn add_green_for_each_creature(g: &mut dyn Game, asker: &Asker, _: Option<Color>) -> Result<(), Suspend> {
    let creatures = g.permanents_of(asker.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).count();
    if creatures > 0 {
        g.add_mana(asker.controller, Mana::G, creatures.min(255) as u8);
    }
    Ok(())
}

/// "This land enters tapped" (CR 614.1c): the modal double-faced lands' (Bala Ged Sanctuary, Haven of the Harvest).
pub fn enters_tapped(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.tapped = true;
    Ok(())
}

/// "As this land enters, you may pay 2 life. If you don't, it enters tapped" (CR 614.12a): the shocklands'. Only a
/// player with the life to pay is asked. Blood Crypt and Breeding Pool print it.
pub fn enters_tapped_unless_pay_two_life(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let paid = g.state().player(entering.controller).life >= 2
        && g.ask(entering.controller, RequestKind::May { source: entering.object, purpose: Purpose::Ask(&PAY_TWO_LIFE_OR_TAPPED) })? == Answer::Yes;
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
        && g.ask(entering.controller, RequestKind::May { source: entering.object, purpose: Purpose::Ask(&PAY_THREE_LIFE_OR_TAPPED) })? == Answer::Yes;
    if paid {
        g.lose_life(entering.controller, 3);
    } else {
        entering.tapped = true;
    }
    Ok(())
}

/// Adds one mana of the color chosen as the ability was activated (CR 106.5), then the source deals 1 damage to its
/// controller, all at once (a mana ability: no stack). The Talismans print it, every one of the five enemy-color pairs:
/// `mana_ability(TAP, add_one_of(&[a, b]).by(mana::add_chosen_and_hurt), text)`.
pub fn add_chosen_and_hurt(g: &mut dyn Game, asker: &Asker, color: Option<Color>) -> Result<(), Suspend> {
    if let Some(c) = color {
        g.add_mana(asker.controller, c.mana(), 1);
    }
    g.damage(asker.source, Target::Player(asker.controller), 1);
    Ok(())
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

/// What a land that may be paid for with 2 life asks as it enters: "Pay 2 life" or "Enters tapped".
pub const PAY_TWO_LIFE_OR_TAPPED: Question =
    Question::new("PayLife", "pay 2 life?", "If you don't, it enters tapped", "Pay 2 life", "Enters tapped").amount(2).no_default();

/// What a land that may be paid for with 3 life asks as it enters: "Pay 3 life" or "Enters tapped".
pub const PAY_THREE_LIFE_OR_TAPPED: Question =
    Question::new("PayLife", "pay 3 life?", "If you don't, it enters tapped", "Pay 3 life", "Enters tapped").amount(3).no_default();

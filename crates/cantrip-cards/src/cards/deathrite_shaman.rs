use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{choose_color, exile_target, targets::card_in_graveyard};

fn land_card(q: &dyn Query, _: &Asker, t: Target) -> bool {
    card_in_graveyard(q, t, &[CardType::Land])
}

fn instant_or_sorcery_card(q: &dyn Query, _: &Asker, t: Target) -> bool {
    card_in_graveyard(q, t, &[CardType::Instant, CardType::Sorcery])
}

fn creature_card(q: &dyn Query, _: &Asker, t: Target) -> bool {
    card_in_graveyard(q, t, &[CardType::Creature])
}

/// The land ability uses the stack (it has a target, so it is no mana ability, CR 605.1a): the color is chosen and
/// the mana added only as it resolves, and not at all if its target is gone.
fn exile_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if exile_target(g, item, 0) {
        let color = choose_color(g, item.controller, item.object)?;
        g.add_mana(item.controller, color.mana(), 1);
    }
    Ok(())
}

fn exile_spell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if exile_target(g, item, 0) {
        g.each_opponent_loses_life(item.controller, 2);
    }
    Ok(())
}

fn exile_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if exile_target(g, item, 0) {
        g.gain_life(item.controller, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP, exile_land, "{T}: Exile target land card from a graveyard. Add one mana of any color.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: land_card }]),
        ability(cost("{B}, {T}"), exile_spell, "{B}, {T}: Exile target instant or sorcery card from a graveyard. Each opponent loses 2 life.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: instant_or_sorcery_card }]),
        ability(cost("{G}, {T}"), exile_creature, "{G}, {T}: Exile target creature card from a graveyard. You gain 2 life.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: creature_card }]),
    ],
    ..oracle::DEATHRITE_SHAMAN
};

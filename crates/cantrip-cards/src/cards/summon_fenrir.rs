use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_battlefield};
use cantrip_mechanics::next_creature_counter;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

/// I: "Search your library for a basic land card, put it onto the battlefield tapped, then shuffle" (CR 701.23).
fn crescent_fang(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_battlefield(g, item.controller, item.object, Find::ONE, true, lands::is_basic_land_card)?;
    Ok(())
}

/// III: "Draw a card if you control the creature with the greatest power or tied for the greatest power": judged as the
/// chapter resolves, among all creatures on the battlefield, Fenrir itself included while it is still there.
fn ecliptic_growl(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<(ObjectId, i16)> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature))
        .map(|id| (id, g.chars(id).power.unwrap_or(0)))
        .collect();
    let greatest = creatures.iter().map(|&(_, power)| power).max();
    if creatures.iter().any(|&(id, power)| Some(power) == greatest && g.controller(id) == item.controller) {
        return effects::draw(g, item, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, crescent_fang, "I \u{2014} Crescent Fang \u{2014} Search your library for a basic land card, put it onto the battlefield tapped, then shuffle."),
        chapter(2, next_creature_counter::chapter, "II \u{2014} Heavenward Howl \u{2014} When you next cast a creature spell this turn, that creature enters with an additional +1/+1 counter on it."),
        chapter(3, ecliptic_growl, "III \u{2014} Ecliptic Growl \u{2014} Draw a card if you control the creature with the greatest power or tied for the greatest power."),
    ],
    delayed: &[next_creature_counter::DELAYED],
    granted_replacements: &[next_creature_counter::ENTERS_WITH_A_COUNTER],
    ..oracle::SUMMON_FENRIR
};

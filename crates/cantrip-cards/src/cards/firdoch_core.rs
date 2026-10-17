use cantrip_api::*;
use cantrip_mechanics::Animation;
use cantrip_oracle as oracle;

/// "{4}: This artifact becomes a 4/4 artifact creature until end of turn." It is an artifact already, and a Kindred
/// and a Shapeshifter, and keeps all of that: the Creature type is added to its types (CR 205.1b, layer 4) and its base
/// power and toughness are set (layer 7b, CR 613.4b), so counters and pumps count on top. Its text names no color and
/// no creature type, so it changes neither. A Core that has left the battlefield is a new object that the ability does
/// not affect (CR 400.7).
///
/// It is the first card with Changeling (CR 702.73a, `Keyword::Changeling`; no other card in `content/` prints it yet).
/// As a Kindred it is every creature type in every zone, so a lord, "choose a creature type", Shoot the Sheriff's
/// Outlaws or a search for a type all see it. Mirror Entity, Universal Automaton and the other changelings need only
/// their content file and their own text.
fn animate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let animation = Animation { power: 4, toughness: 4, colors: None, subtype: None, all_creature_types: false, keywords: Keywords::NONE, ability: None };
    cantrip_mechanics::animate(g, item.object, item.object, &animation, Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        ability(cost("{4}"), animate, "{4}: This artifact becomes a 4/4 artifact creature until end of turn."),
    ],
    ..oracle::FIRDOCH_CORE
};

use cantrip_api::*;
use cantrip_mechanics::damage_target;
use cantrip_mechanics::{attach::attached_object, targets};
use cantrip_oracle as oracle;

/// "Target player or planeswalker" (CR 115.4): any target narrowed to players and planeswalkers.
fn player_or_planeswalker(q: &dyn Query, _: &Asker, t: Target) -> bool {
    match t {
        Target::Player(_) => true,
        Target::Object(id) => targets::permanent(q, id) && q.chars(id).is(CardType::Planeswalker),
        Target::None | Target::Ability(_) => false,
    }
}

/// "This creature deals 1 damage to target player or planeswalker. You draw a card." The creature is the source, and
/// "you" is whoever controls it.
fn ping_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    g.draw(item.controller, 1);
    Ok(())
}

/// The enchanted creature has the ability (layer 6, CR 613.1f): the Aura's delayed ability 0, an ability the creature
/// controls and that is gone the moment the Aura stops enchanting it.
fn grant_ping(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    delayed: &[ability(TAP, ping_and_draw, "{T}: This creature deals 1 damage to target player or planeswalker. You draw a card.")
        .targets(&[TargetSpec { min: 1, max: 1, filter: player_or_planeswalker }])],
    statics: &[StaticDef {
        text: "Enchanted creature has \"{T}: This creature deals 1 damage to target player or planeswalker. You draw a card.\"",
        layer: Layer::Ability,
        affects: attached_object,
        apply: grant_ping,
        ..StaticDef::BLANK
    }],
    ..oracle::QUICKSILVER_DAGGER
};

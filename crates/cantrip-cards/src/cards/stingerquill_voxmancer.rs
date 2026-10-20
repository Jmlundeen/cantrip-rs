use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::YOUR_UPKEEP;
use cantrip_oracle as oracle;

/// "Vicious Verse deals 1 damage to target opponent": a target no longer legal is dealt nothing (CR 608.2b).
fn vicious_verse(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    g.damage(item.object, Target::Player(opponent), 1);
    Ok(())
}

/// Stingerquill Voxmancer's prepare spell (CR 722): not a card, only ever a copy cast from exile (CR 722.3c).
pub static VICIOUS_VERSE: CardDef = CardDef {
    name: "Vicious Verse",
    cost: mana("{B/R}"),
    types: TypeSet::of(&[CardType::Sorcery]),
    spell: spell(&[targets::OPPONENT], vicious_verse),
    text: "Vicious Verse deals 1 damage to target opponent.",
    ..CardDef::BLANK
};

/// The intervening "if this creature isn't prepared" (CR 603.4), checked as the ability triggers and as it resolves.
fn this_is_not_prepared(q: &dyn Query, me: &Asker) -> bool {
    !q.obj(me.source).prepared
}

/// "It becomes prepared" (CR 722.3a): nothing if it left the battlefield, or already is prepared.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_UPKEEP,
        cantrip_mechanics::prepare::become_prepared,
        "At the beginning of your upkeep, if this creature isn't prepared, it becomes prepared. (While it's prepared, you may cast a copy of its spell. Doing so unprepares it.)",
    )
    .condition(this_is_not_prepared)],
    prepare: Some(&VICIOUS_VERSE),
    ..oracle::STINGERQUILL_VOXMANCER
};

pub static EXTRA: &[&CardDef] = &[&VICIOUS_VERSE];

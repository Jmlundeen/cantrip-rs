use cantrip_api::*;
use cantrip_mechanics::{choose, effects, select, statics, triggers};
use cantrip_oracle as oracle;

/// "You may sacrifice a creature or planeswalker" (CR 701.21a): the chosen permanent must be one you control.
fn creature_or_planeswalker(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |card| {
        let chars = q.chars(card);
        chars.is(CardType::Creature) || chars.is(CardType::Planeswalker)
    })
}

/// "When Winter enters, you may sacrifice a creature or planeswalker. When you do, ..." (CR 603.12).
fn sacrifice_for_edict(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = Selection::new(select::up_to(1))
        .with(select::part(&(creature_or_planeswalker as SelectFn), 0, ""))
        .called("creature or planeswalker");
    let yours = g.permanents_of(item.controller);
    let chosen = choose::select(g, item.controller, item.object, Purpose::Sacrifice, yours, rule)?;
    if !g.sacrifice(item.controller, &chosen).is_empty() {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

/// "Winter gets +1/+0 for each creature and planeswalker card in your graveyard" (CR 613.4c).
/// A card with both types is counted once.
fn graveyard_power(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let state = q.state();
    let n = state.zone(Zone::graveyard(state.obj(source).controller)).iter().filter(|&&card| {
        let obj = state.obj(card);
        let chars = q.chars(card);
        !obj.token && !obj.card_copy && (chars.is(CardType::Creature) || chars.is(CardType::Planeswalker))
    }).count() as i16;
    chars.power = chars.power.map(|power| power + n);
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, sacrifice_for_edict,
        "When Winter enters, you may sacrifice a creature or planeswalker. When you do, each opponent sacrifices a creature of their choice.")],
    delayed: &[delayed_trigger(effects::each_opponent_sacrifices_a_creature, "When you do, each opponent sacrifices a creature of their choice.")],
    statics: &[static_ability(Layer::PtModify, statics::itself, graveyard_power,
        "Winter gets +1/+0 for each creature and planeswalker card in your graveyard.")],
    ..oracle::WINTER_TORMENTED_LONER
};

use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::triggers::YOU_GAIN_LIFE;
use cantrip_oracle as oracle;

/// "Whenever you gain life, put a charge counter on [this]" (CR 122.1, 603.2).
fn put_a_charge_counter_on_this(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Charge, 1);
    Ok(())
}

/// "Equipped creature gets +1/+1 for each charge counter on [this]" (CR 613.4c): counted as the layer stands, so counters
/// put on it mid-turn count at once.
fn plus_one_per_charge_counter(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let n = view.state().obj(source).counters(CounterKind::Charge);
    chars.power = chars.power.map(|v| v + n);
    chars.toughness = chars.toughness.map(|v| v + n);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOU_GAIN_LIFE, put_a_charge_counter_on_this, "Whenever you gain life, put a charge counter on Excalibur II."),
        equip(mana("{3}"), "Equip {3}"),
    ],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_one_per_charge_counter, "Equipped creature gets +1/+1 for each charge counter on Excalibur II.")],
    ..oracle::EXCALIBUR_II
};

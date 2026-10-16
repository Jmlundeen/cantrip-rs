use cantrip_api::*;
use cantrip_mechanics::attach::attached_object;
use cantrip_mechanics::reconfigure;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{W}: Exile target card from a graveyard. If it was a permanent card, put a +1/+1 counter on this permanent."
fn exile_from_graveyard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let permanent = targets::is_permanent_card(&g.chars(card));
    g.exile(card);
    if permanent && targets::permanent(g, item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// "Equipped creature gets +1/+1 for each +1/+1 counter on this Equipment."
fn per_counter(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let n = q.state().obj(source).counters(CounterKind::PlusOne).max(0);
    chars.power = chars.power.map(|v| v + n);
    chars.toughness = chars.toughness.map(|v| v + n);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{W}"), exile_from_graveyard, "{W}: Exile target card from a graveyard. If it was a permanent card, put a +1/+1 counter on this permanent.")
            .targets(&[targets::CARD_IN_A_GRAVEYARD]),
        reconfigure::attach(mana("{2}"), "Reconfigure {2} ({2}: Attach this permanent to another target creature you control. Reconfigure only as a sorcery.)"),
        reconfigure::unattach(mana("{2}"), "Reconfigure {2} ({2}: Unattach this permanent. Reconfigure only if it is attached to a creature and only as a sorcery.)"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, per_counter, "Equipped creature gets +1/+1 for each +1/+1 counter on this Equipment."),
        reconfigure::NOT_A_CREATURE_WHILE_ATTACHED,
    ],
    reconfigure: true,
    ..oracle::LION_SASH
};

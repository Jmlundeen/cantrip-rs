use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

/// "As long as there are three or more different kinds of counters among creatures you control, this creature gets +2/+4"
/// (CR 613.4c, 122.1): itself, while the creatures its controller controls (as the earlier layers made them) carry counters of
/// at least three kinds between them.
fn itself_with_three_kinds_of_counters(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    let state = view.state();
    let you = state.obj(source).controller;
    affected == source
        && CounterKind::ALL
            .iter()
            .filter(|&&kind| {
                state.battlefield().iter().any(|&id| state.obj(id).controller == you && view.chars(id).is(CardType::Creature) && state.obj(id).counters(kind) > 0)
            })
            .count()
            >= 3
}

/// "You may cast this card from your graveyard" (CR 601.2, 404.1): this card, by its owner.
fn this_card_by_its_owner(_: &dyn Query, asker: &Asker, card: ObjectId, player: PlayerId) -> bool {
    card == asker.source && player == asker.controller
}

/// "You may cast this card from your graveyard. If you do, it enters with a finality counter on it" (CR 122.1, 614.1c, 601.3):
/// when the spell was cast by this card's own permission. A cast from the graveyard by another permission (Yawgmoth's Will) is
/// not "if you do", so it enters without one.
fn finality_if_cast_by_its_permission(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    let own = AbilityRef::exile_cast(g.obj(entering.object).card, 0);
    if history::cast_of(g, entering.object).and_then(|cast| cast.granted_by()).is_some_and(|by| by.ability == own) {
        entering.counters[CounterKind::Finality as usize] += 1;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(finality_if_cast_by_its_permission),
    statics: &[static_ability(
        Layer::PtModify,
        itself_with_three_kinds_of_counters,
        plus_pt::<2, 4>,
        "As long as there are three or more different kinds of counters among creatures you control, this creature gets +2/+4.",
    )],
    exile_casts: &[ExileCast {
        text: "Cast from your graveyard",
        from: ZoneKind::Graveyard,
        allows: this_card_by_its_owner,
        additional: Cost::NONE,
        any_type: false,
        any_color: false,
        lands: false,
        life_for_mana_value: false,
        instead: None,
    }],
    ..oracle::HUNDRED_BATTLE_VETERAN
};

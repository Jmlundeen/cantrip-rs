use cantrip_api::*;
use cantrip_mechanics::triggers::{self, THIS_ENTERS};
use cantrip_mechanics::{choose, effects, history, select, targets};
use cantrip_oracle as oracle;

fn creature_card_with_mana_value_1_or_less_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard_with_mana_value_at_most(q, asker, t, &[CardType::Creature], 1)
}

/// "Return target creature card with mana value 1 or less from your graveyard to the battlefield" (CR 400.7, 608.2b): under its
/// owner's control, and nothing happens if the target is no longer there.
fn return_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::return_first_target_to_battlefield(g, item)
}

/// "Whenever Venom attacks, you may sacrifice another creature. If you do, draw X cards, then you may put a permanent card with
/// mana value X or less from your hand onto the battlefield, where X is the sacrificed creature's mana value." (CR 701.21a,
/// 608.2c, 608.2h, 202.3.) X is the mana value the creature last had on the battlefield; the hand is judged after the draw, a
/// card in a hand has no X in its cost (CR 202.3e), and a permanent card is any card that is not an instant or a sorcery (CR 110.4a).
fn sacrifice_draw_put(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let Some(&sacrificed) = effects::may_sacrifice(g, you, item.object, select::another_creature_permanents(select::up_to(1)))?.objects().first() else {
        return Ok(());
    };
    g.end_sentence();
    let x = history::mana_value_of_sacrificed_permanent(g, sacrificed);
    g.draw(you, x);
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    let rule = select::cards(select::up_to(1)).with(select::PERMANENT_CARD).with(select::mana_value_at_most(x));
    let chosen = choose::select(g, you, item.object, Purpose::PutOntoBattlefield, hand, rule)?;
    for card in chosen {
        g.put_onto_battlefield(card, you, false)?;
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ATTACKS,
        sacrifice_draw_put,
        "Whenever Venom attacks, you may sacrifice another creature. If you do, draw X cards, then you may put a permanent card with mana value X or less from your hand onto the battlefield, where X is the sacrificed creature's mana value.",
    )],
    ..oracle::VENOM_LETHAL_PROTECTOR
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            return_target,
            "When Eddie Brock enters, return target creature card with mana value 1 or less from your graveyard to the battlefield.",
        )
        .targets(&[target(creature_card_with_mana_value_1_or_less_in_your_graveyard)]),
        ability(cost("{3}{B}{R}{G}"), effects::transform_self, "{3}{B}{R}{G}: Transform Eddie Brock. Activate only as a sorcery.").sorcery_speed(),
    ],
    back: Some(&BACK),
    ..oracle::EDDIE_BROCK
};

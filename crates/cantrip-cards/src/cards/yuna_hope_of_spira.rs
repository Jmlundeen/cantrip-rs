use cantrip_api::*;
use cantrip_mechanics::statics::grant_delayed_ability;
use cantrip_mechanics::triggers::{ward, YOUR_END_STEP};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "During your turn, Yuna and enchantment creatures you control" (CR 604.1, 613.1f): this creature itself, and each
/// enchantment creature its controller controls, as the earlier layers made it, while its controller is the active player.
fn yuna_and_your_enchantment_creatures_on_your_turn(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let s = q.state();
    let me = s.obj(source).controller;
    s.turn.active == me
        && (affected == source
            || (s.obj(affected).controller == me && so_far.is(CardType::Creature) && so_far.is(CardType::Enchantment)))
}

/// "... have trample, lifelink, and ward {2}" (CR 702.19, 702.15, 702.21, layer 6): the first two keywords, and the ward this
/// card lists as its delayed ability.
fn have_trample_lifelink_and_ward(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample).with(Keyword::Lifelink);
    grant_delayed_ability(view, source, chars);
}

/// "Target enchantment card from your graveyard" (CR 115.1, 400.7).
fn enchantment_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Enchantment])
}

/// "Return up to one target enchantment card from your graveyard to the battlefield with a finality counter on it": the
/// counter is on it as it enters (CR 122.6, 614.1c).
fn return_with_finality(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield_with_counters(card, item.controller, false, &[(CounterKind::Finality, 1)])?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        return_with_finality,
        "At the beginning of your end step, return up to one target enchantment card from your graveyard to the battlefield with a finality counter on it.",
    )
    .targets(&[target(enchantment_card_in_your_graveyard).between(0, 1)])],
    delayed: &[ward::<2>("Ward {2} (Whenever this permanent becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)")],
    statics: &[
        static_ability(
            Layer::Ability,
            yuna_and_your_enchantment_creatures_on_your_turn,
            have_trample_lifelink_and_ward,
            "During your turn, Yuna and enchantment creatures you control have trample, lifelink, and ward {2}.",
        ),
    ],
    ..oracle::YUNA_HOPE_OF_SPIRA
};

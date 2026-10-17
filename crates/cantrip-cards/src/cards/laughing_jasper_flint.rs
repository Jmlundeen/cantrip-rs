use cantrip_api::*;
use cantrip_mechanics::library::exile_top_may_play;
use cantrip_mechanics::statics::your_creatures;
use cantrip_mechanics::{outlaw, targets, triggers::YOUR_UPKEEP};
use cantrip_oracle as oracle;

/// "Creatures you control but don't own" (CR 613.1d, layer 4): a creature, as the earlier layers made it, under the source's
/// controller's control whose owner is another player. A token is owned by the player who made it (CR 111.2), so one of yours
/// is not.
fn creatures_you_control_but_dont_own(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    your_creatures(q, source, affected, so_far) && q.state().obj(affected).owner != q.state().obj(source).controller
}

/// "... are Mercenaries in addition to their other types."
fn mercenary_too(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.subtypes = chars.subtypes.with(view.words(source).subtype(Subtype::Mercenary));
}

/// "Exile the top X cards of target opponent's library, where X is the number of outlaws you control. Until end of turn, you may cast
/// spells from among those cards, and mana of any type can be spent to cast those spells." (CR 609.4b, 601.3, 400.7) X is
/// counted as the ability resolves; a land among the cards cannot be cast, so it is not played; the permission ends at the
/// cleanup step (CR 514.2) and the cards stay in exile.
fn exile_and_cast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.targets[0].player() else { return Ok(()) };
    let x = outlaw::outlaws_controlled_by(g, item.controller);
    exile_top_may_play(g, opponent, x, item.controller, PlayGrant { any_type: true, expiry: Expiry::EndOfTurn, ..PlayGrant::CAST });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Type, creatures_you_control_but_dont_own, mercenary_too, "Creatures you control but don't own are Mercenaries in addition to their other types.")],
    abilities: &[triggered(YOUR_UPKEEP, exile_and_cast, "At the beginning of your upkeep, exile the top X cards of target opponent's library, where X is the number of outlaws you control. Until end of turn, you may cast spells from among those cards, and mana of any type can be spent to cast those spells.")
        .targets(&[targets::OPPONENT])],
    ..oracle::LAUGHING_JASPER_FLINT
};

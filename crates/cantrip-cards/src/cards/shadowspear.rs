use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// "... and has trample and lifelink."
fn trample_and_lifelink(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample).with(Keyword::Lifelink);
}

/// "{1}: Permanents your opponents control lose hexproof and indestructible until end of turn." The permanents are those
/// opponents control as the ability resolves, and only those (CR 611.2c): one that arrives later keeps both.
fn opponents_permanents_lose_protection(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let lost = Keywords::of(&[Keyword::Hexproof, Keyword::Indestructible]);
    let opponents: Vec<PlayerId> = g.state().opponents(item.controller).collect();
    let permanents: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| opponents.contains(&g.controller(id))).collect();
    instruction(g, |g| {
        for permanent in permanents {
            g.add_effect(item.object, permanent, EffectKind::LoseKeywords(lost), Duration::EndOfTurn);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}"), opponents_permanents_lose_protection, "{1}: Permanents your opponents control lose hexproof and indestructible until end of turn."),
        equip(mana("{2}"), "Equip {2}"),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, 1>, "Equipped creature gets +1/+1."),
        static_ability(Layer::Ability, attached_object, trample_and_lifelink, "Equipped creature has trample and lifelink."),
    ],
    ..oracle::SHADOWSPEAR
};

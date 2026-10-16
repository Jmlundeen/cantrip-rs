use cantrip_api::*;
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_oracle as oracle;

/// "Equipped creature gets +1/+1" (CR 301.5).
fn plus_one_plus_one(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
    chars.toughness = chars.toughness.map(|v| v + 1);
}

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
    for permanent in permanents {
        g.add_effect(item.object, permanent, EffectKind::LoseKeywords(lost), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}"), opponents_permanents_lose_protection, "{1}: Permanents your opponents control lose hexproof and indestructible until end of turn."),
        equip(mana(2, 0, 0, 0, 0, 0), "Equip {2}"),
    ],
    statics: &[
        StaticDef {
            text: "Equipped creature gets +1/+1.",
            layer: Layer::PtModify,
            affects: attached_object,
            apply: plus_one_plus_one,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Equipped creature has trample and lifelink.",
            layer: Layer::Ability,
            affects: attached_object,
            apply: trample_and_lifelink,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::SHADOWSPEAR
};

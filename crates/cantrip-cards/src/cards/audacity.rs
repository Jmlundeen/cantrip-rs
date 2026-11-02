use cantrip_api::*;
use cantrip_mechanics::{attach, targets, triggers};
use cantrip_oracle as oracle;

/// "Enchanted creature gets +2/+0 and has trample." (CR 604.1).
fn enchanted_creature_gets_2_0_apply_1(
    _: &dyn LayerQuery,
    _: ObjectId,
    chars: &mut Chars,
) {
    chars.keywords = chars.keywords.with(Keyword::Trample);
}

/// "Enchanted creature gets +2/+0 and has trample." (CR 604.1).
fn enchanted_creature_gets_2_0_apply_0(
    _: &dyn LayerQuery,
    _: ObjectId,
    chars: &mut Chars,
) {
    let (p, t) = (2i16, 0i16);
    chars.power = chars.power.map(|v| v.saturating_add(p));
    chars.toughness = chars.toughness.map(|v| v.saturating_add(t));
}

/// "When this Aura is put into a graveyard from the battlefield, draw a card." (CR 608.2, 121.1).
fn draw_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_DIES,
            draw_card,
            "When this Aura is put into a graveyard from the battlefield, draw a card.",
        ),
    ],
    statics: &[
        static_ability(
            Layer::Ability,
            attach::attached_object,
            enchanted_creature_gets_2_0_apply_1,
            "Enchanted creature gets +2/+0 and has trample.",
        ),
        static_ability(
                Layer::PtModify,
                attach::attached_object,
                enchanted_creature_gets_2_0_apply_0,
                "Enchanted creature gets +2/+0 and has trample.",
            )
            .continues(),
    ],
    enchant: Some(targets::CREATURE),
    ..oracle::AUDACITY
};

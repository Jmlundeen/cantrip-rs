use cantrip_api::*;
use cantrip_mechanics::{flurry, tokens};
use cantrip_mechanics::attach::{attached_object, equip};
use cantrip_mechanics::tokens::MONK;
use cantrip_oracle as oracle;

fn plus_one_plus_one(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 1);
    chars.toughness = chars.toughness.map(|v| v + 1);
}

fn trample_and_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample).with(Keyword::Haste);
}

/// Create the Monk; the Equipment can be offered only if it is still on the battlefield and the token entered
/// (CR 701.3b), and then it may be attached to it.
fn make_monk(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let made = tokens::create(g, item.controller, &MONK, 1)?;
    let equipment = item.object;
    let Some(&token) = made.first() else { return Ok(()) };
    if !is_live(g, equipment) || !g.is_on_battlefield(token) {
        return Ok(());
    }
    if cantrip_mechanics::effects::ask_may(g, item.controller, equipment, Purpose::Optional)? {
        g.attach(equipment, token);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        equip(mana(1, 0, 0, 0, 1, 0), "Equip {1}{R}"),
        flurry::trigger(
            "Flurry \u{2014} Whenever you cast your second spell each turn, create a 1/1 white Monk creature token with prowess. You may attach this Equipment to it.",
            make_monk,
        ),
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
            text: "Equipped creature has trample and haste.",
            layer: Layer::Ability,
            affects: attached_object,
            apply: trample_and_haste,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::CORI_STEEL_CUTTER
};

pub static EXTRA: &[&CardDef] = &[&MONK];

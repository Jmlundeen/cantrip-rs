use cantrip_api::*;
use cantrip_mechanics::{flurry, tokens};
use cantrip_mechanics::attach::{self, attached_object, equip};
use cantrip_mechanics::tokens::MONK;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Attach", "attach it to the Monk token?", "", "Attach it", "Don't attach");

fn trample_and_haste(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::Trample).with(Keyword::Haste);
}

/// Create the Monk; the Equipment can be offered only if it is still on the battlefield and the token entered
/// (CR 701.3b), and then it may be attached to it.
fn make_monk(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let made = tokens::create(g, item.controller, &MONK, 1)?;
    let Some(&token) = made.first() else { return Ok(()) };
    attach::may_attach_to(g, item.controller, item.object, token, Purpose::Ask(&ASK))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        equip(mana("{1}{R}"), "Equip {1}{R}"),
        flurry::trigger(
            "Flurry \u{2014} Whenever you cast your second spell each turn, create a 1/1 white Monk creature token with prowess. You may attach this Equipment to it.",
            make_monk,
        ),
    ],
    statics: &[
        static_ability(Layer::PtModify, attached_object, cantrip_mechanics::statics::plus_pt::<1, 1>, "Equipped creature gets +1/+1."),
        static_ability(Layer::Ability, attached_object, trample_and_haste, "Equipped creature has trample and haste."),
    ],
    ..oracle::CORI_STEEL_CUTTER
};

pub static EXTRA: &[&CardDef] = &[&MONK];

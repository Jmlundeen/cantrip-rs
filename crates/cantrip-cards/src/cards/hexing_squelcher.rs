use cantrip_api::*;
use cantrip_mechanics::triggers::ward_life;
use cantrip_oracle as oracle;

const WARD: AbilityDef = ward_life::<2>("Ward\u{2014}Pay 2 life. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays 2 life.)");

/// "Spells you control can't be countered": spells on the stack (a card on the stack has card types, an ability does not).
fn your_spells(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    let o = q.state().obj(affected);
    o.zone.kind() == ZoneKind::Stack && so_far.types != TypeSet::NONE && o.controller == q.state().obj(source).controller
}

fn uncounterable(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::CantBeCountered);
}

fn other_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    affected != source && so_far.is(CardType::Creature) && q.state().obj(affected).controller == q.state().obj(source).controller
}

fn grant_ward(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[WARD],
    delayed: &[WARD],
    statics: &[
        StaticDef {
            text: "Spells you control can't be countered.",
            layer: Layer::Ability,
            affects: your_spells,
            apply: uncounterable,
            everywhere: true,
            ..StaticDef::BLANK
        },
        StaticDef {
            text: "Other creatures you control have \"Ward\u{2014}Pay 2 life.\"",
            layer: Layer::Ability,
            affects: other_creatures_you_control,
            apply: grant_ward,
            ..StaticDef::BLANK
        },
    ],
    ..oracle::HEXING_SQUELCHER
};

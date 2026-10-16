use cantrip_api::*;
use cantrip_mechanics::triggers::NONCREATURE_SPELL_YOU_CAST;
use cantrip_oracle as oracle;

/// Delirium (an ability word, CR 207.2c): four or more card types among cards in the controller's graveyard.
fn delirium(q: &dyn LayerQuery, source: ObjectId) -> bool {
    let state = q.state();
    let you = state.obj(source).controller;
    let mut seen = TypeSet::NONE;
    for &card in state.zone(Zone::graveyard(you)) {
        for t in q.chars(card).types.iter() {
            seen = seen.with(t);
        }
    }
    seen.iter().count() >= 4
}

/// Only the Channeler itself, on the battlefield, while delirium holds.
fn affects(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && delirium(q, source)
}

fn plus_two_plus_two(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.power = chars.power.map(|v| v + 2);
    chars.toughness = chars.toughness.map(|v| v + 2);
}

fn must_attack(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.must_attack = true;
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(NONCREATURE_SPELL_YOU_CAST, |g, item| cantrip_mechanics::effects::surveil(g, item, 1), "Whenever you cast a noncreature spell, surveil 1.")],
    statics: &[
        static_ability(Layer::PtModify, affects, plus_two_plus_two, "Delirium \u{2014} As long as there are four or more card types among cards in your graveyard, this creature gets +2/+2."),
        static_ability(Layer::Ability, affects, cantrip_mechanics::statics::have_flying, "Delirium \u{2014} As long as there are four or more card types among cards in your graveyard, this creature has flying."),
        static_ability(Layer::Ability, affects, must_attack, "Delirium \u{2014} As long as there are four or more card types among cards in your graveyard, this creature attacks each combat if able."),
    ],
    ..oracle::DRAGONS_RAGE_CHANNELER
};

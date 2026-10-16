use cantrip_api::*;
use cantrip_mechanics::combat::{choose_among, create_attacking, player_or_their_planeswalkers};
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// The 1/1 white Human creature token it makes.
pub static HUMAN: CardDef = CardDef {
    name: "Human",
    has_cost: false,
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    subtypes: SubtypeSet::of(&[Subtype::Human]),
    power: Some(1),
    toughness: Some(1),
    ..CardDef::BLANK
};

/// "Adeline's power is equal to the number of creatures you control" (CR 604.3, layer 7a): a characteristic-defining
/// ability, itself included.
fn power_is_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let controller = q.state().obj(source).controller;
    let count = q
        .state()
        .battlefield()
        .iter()
        .filter(|&&id| q.state().obj(id).controller == controller && q.chars(id).is(CardType::Creature))
        .count();
    chars.power = Some(count as i16);
}

/// "For each opponent, create a 1/1 white Human creature token that's tapped and attacking that player or a planeswalker
/// they control" (CR 508.4): its controller chooses which, asked only when the opponent controls a planeswalker.
fn human_for_each_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let human = g.card_id(&HUMAN);
    let opponents: Vec<PlayerId> = g.state().opponents(item.controller).collect();
    let mut chosen: Vec<Target> = Vec::new();
    for opponent in opponents {
        let candidates = player_or_their_planeswalkers(g, opponent);
        chosen.extend(choose_among(g, item.controller, item.object, &candidates)?);
    }
    create_attacking(g, item.controller, human, &chosen)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[StaticDef {
        text: "Adeline's power is equal to the number of creatures you control.",
        layer: Layer::PtDefine,
        apply: power_is_creatures_you_control,
        own: true,
        ..StaticDef::BLANK
    }],
    abilities: &[triggered(triggers::YOU_ATTACK, human_for_each_opponent, "Whenever you attack, for each opponent, create a 1/1 white Human creature token that's tapped and attacking that player or a planeswalker they control.")],
    ..oracle::ADELINE_RESPLENDENT_CATHAR
};

pub static EXTRA: &[&CardDef] = &[&HUMAN];

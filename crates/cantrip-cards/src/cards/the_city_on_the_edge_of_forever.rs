use cantrip_api::*;
use cantrip_mechanics::{effects, sacrifice_one_of, tokens};
use cantrip_oracle as oracle;

/// "Permanents can't phase in." (CR 702.26a, 101.2): every permanent that would phase in, whoever controls it and whatever
/// it is. A rule of the game, asked of the permanent that would phase in (`CardDef::phase_in_bans`).
fn cant_phase_in(_: &dyn Query, _: &Asker, _: ObjectId) -> bool {
    true
}

/// The 0/1 white Human creature token with "Permanents can't phase in." (CR 111.10x); The City on the Edge of Forever is the
/// only printed card that makes it.
pub static HUMAN: CardDef = CardDef {
    colors: Some(ColorSet::of(Color::White)),
    types: TypeSet::of(&[CardType::Creature]),
    power: Some(0),
    toughness: Some(1),
    phase_in_bans: &[cant_phase_in],
    text: "Permanents can't phase in.",
    ..token!(Human)
};

/// Chapter I: "All artifacts and creatures phase out. Create a 0/1 white Human creature token with 'Permanents can't phase
/// in.'" (CR 702.26b, 608.2): every artifact and creature on the battlefield, the Saga's own (it is neither), phases out at
/// once; the token comes after, so what phased out can never phase back in.
fn i_phase_out(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let out: Vec<ObjectId> = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| q.chars(id).is(CardType::Artifact) || q.chars(id).is(CardType::Creature))
            .collect()
    };
    g.phase_out(&out);
    tokens::create(g, item.controller, &HUMAN, 1)?;
    Ok(())
}

/// Chapter III: "Sacrifice a creature token. If you do, you gain 5 life." (CR 701.21a, 608.2c): not optional - a creature
/// token you control is sacrificed whenever there is one (chosen as the chapter resolves, not as a cost), and the life is
/// gained only if one actually was.
fn iii_sacrifice_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let tokens: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.obj(id).token && g.chars(id).is(CardType::Creature)).collect();
    if !sacrifice_one_of(g, item.controller, item.object, 1, tokens)?.is_empty() {
        g.gain_life(item.controller, 5);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(
            1,
            i_phase_out,
            "I \u{2014} All artifacts and creatures phase out. Create a 0/1 white Human creature token with \"Permanents can't phase in.\"",
        ),
        chapter(2, effects::draw_two_each_player_loses_two, "II \u{2014} You draw two cards and each player loses 2 life."),
        chapter(3, iii_sacrifice_and_gain, "III \u{2014} Sacrifice a creature token. If you do, you gain 5 life."),
    ],
    ..oracle::THE_CITY_ON_THE_EDGE_OF_FOREVER
};

pub static EXTRA: &[&CardDef] = &[&HUMAN];

use cantrip_api::*;
use cantrip_mechanics::{lands, statics, tokens, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your end step, each player creates two 1/1 white Tribble creature tokens with 'This token can't
/// block.'" (CR 603.3b, 111.1, 101.4): each player in turn order starting with the active player makes two, each under
/// their own control, together as one event; a player who has left the game is not "each player" (CR 800.4a). The token is
/// `tokens::TRIBBLE`, defined once for every card that makes one.
fn each_player_creates_two_tribbles(g: &mut dyn Game, _item: &StackItem) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    for player in players {
        tokens::create(g, player, &tokens::TRIBBLE, 2)?;
    }
    Ok(())
}

const TRIBBIES: &str = "All Tribbles get +1/+0 and are goaded. (They attack each combat if able and attack a player other than you if able.)";

/// "All Tribbles" (CR 701.15a): every permanent with the creature type Tribble, whoever controls it, judged as the earlier
/// layers left it (a Conspiracy naming Tribble makes a creature one). The goader is this source's controller (CR 701.15b).
fn all_tribbles(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    q.chars(id).has_subtype(q.words(asker.source).subtype(Subtype::Tribble))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_END_STEP,
        each_player_creates_two_tribbles,
        "At the beginning of your end step, each player creates two 1/1 white Tribble creature tokens with \"This token can't block.\"",
    )],
    statics: &[static_ability(
        Layer::PtModify,
        statics::all_of_subtype::<{ lands::sub(Subtype::Tribble) }>,
        statics::plus_pt::<1, 0>,
        TRIBBIES,
    )],
    goads: &[all_tribbles],
    ..oracle::KOLOTH_TRIBBLE_HUNTER
};

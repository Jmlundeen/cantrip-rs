use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Destroy target permanent. Its controller creates a 3/3 green Beast creature token." (CR 608.2, 701.8a).
fn destroy_target_permanent_controller_creates(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let t_0 = vec![g.state().target(item, 0)];
    let objects_0 = t_0
        .get(0)
        .copied()
        .and_then(|t: Target| t.object())
        .into_iter()
        .collect::<Vec<_>>();
    g.destroy_all(&objects_0);
    g.end_sentence();
    let players_1 = {
        let q: &dyn Query = g;
        t_0.get(0)
            .copied()
            .and_then(|t: Target| t.object())
            .map(|o: ObjectId| q.controller(o))
            .into_iter()
            .collect::<Vec<PlayerId>>()
    };
    let mut tokens_1 = Vec::new();
    let def_1_0 = g.card_id(&tokens::BEAST_3_3);
    for player in players_1 {
        tokens_1.extend(std::iter::repeat((def_1_0, player)).take(1));
    }
    let words_1 = g.words(item.object);
    g.create_tokens_worded(&tokens_1, false, None, words_1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PERMANENT], destroy_target_permanent_controller_creates),
    ..oracle::BEAST_WITHIN
};

pub static EXTRA: &[&CardDef] = &[&tokens::BEAST_3_3];

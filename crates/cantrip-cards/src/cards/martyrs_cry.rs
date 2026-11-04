use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Exile all white creatures. For each creature exiled this way, its controller draws a card." (CR 701.13a, 608.2h, 121.2c).
fn exile_all_white_creatures_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = {
        let q: &dyn Query = g;
        q.state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| q.chars(id).colors.has(q.words(item.object).color(Color::White)) && q.chars(id).is(CardType::Creature))
            .collect()
    };
    let exiled = g.exile_all(&creatures);
    g.end_sentence();
    let players: Vec<PlayerId> = g.state().apnap().collect();
    for p in players {
        let n = {
            let q: &dyn Query = g;
            creatures
                .iter()
                .filter(|&&o| q.state().obj(o).became.is_some_and(|b| exiled.contains(&b)) && q.controller(o) == p)
                .count() as u32
        };
        g.draw(p, n);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], exile_all_white_creatures_creature),
    ..oracle::MARTYRS_CRY
};

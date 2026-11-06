use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You may look at the top card of your library any time. You may play lands from the top of your library. Coven — As long as you control three or more creatures with different powers, you may cast creature spells from the top of your library." (CR 604.1).
fn look_top_card_library_any_rule(q: &dyn Query, _: &Asker, spell: ObjectId) -> bool {
    q.chars(spell).is(CardType::Creature)
}

/// "You may look at the top card of your library any time. You may play lands from the top of your library. Coven — As long as you control three or more creatures with different powers, you may cast creature spells from the top of your library." (CR 608.2).
fn look_top_card_library_any_while(q: &dyn Query, asker: &Asker) -> bool {
    ({
        let mut seen = Vec::new();
        for id in q
            .state()
            .battlefield()
            .iter()
            .copied()
            .filter(|&id| {
                q.chars(id).is(CardType::Creature)
                    && asker.controller == q.controller(id)
            })
            .collect::<Vec<ObjectId>>()
        {
            let value = q.chars(id).power;
            if !seen.contains(&value) {
                seen.push(value);
            }
        }
        seen.len() as i64
    }) >= 3
}

pub static CARD: CardDef = CardDef {
    play_rules: &[
        PlayRule::LookAtLibraryTop,
        PlayRule::LibraryTopLands,
        PlayRule::While(
            look_top_card_library_any_while,
            &[PlayRule::LibraryTopSpells(look_top_card_library_any_rule, None)],
        ),
    ],
    ..oracle::AUGUR_OF_AUTUMN
};

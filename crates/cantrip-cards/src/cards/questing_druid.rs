use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::library::exile_top_play_until_your_next_end_step;
use cantrip_oracle as oracle;

/// The Adventure (CR 715): "Exile the top two cards of your library. Until your next end step, you may play those cards." A
/// permission to play, so a land may be the turn's land drop (CR 305.1).
fn seek_the_beast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_top_play_until_your_next_end_step(g, item.controller, 2);
    Ok(())
}

pub static SEEK_THE_BEAST: CardDef = CardDef { spell: spell(&[], seek_the_beast), ..oracle::SEEK_THE_BEAST };

/// "Whenever you cast a spell that's white, blue, black, or red" (CR 603.2, 105.2): one trigger for a spell of any of the
/// four colors, however many of them it is. Judged on the spell as it is on the stack (CR 601.2i), so the Adventure side of
/// a card counts by its own colors. The color words are words of the Druid's text (CR 612).
fn you_cast_a_white_blue_black_or_red_spell(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::SpellCast { spell, player, .. } if *player == q.controller(source) => {
            let words = q.words(source);
            let colors = q.chars(*spell).colors;
            if [Color::White, Color::Blue, Color::Black, Color::Red].into_iter().any(|c| colors.has(words.color(c))) {
                Fired::PLAIN
            } else {
                None
            }
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::SPELL_CAST, you_cast_a_white_blue_black_or_red_spell),
        plus_one_counter_on_this,
        "Whenever you cast a spell that's white, blue, black, or red, put a +1/+1 counter on this creature.",
    )],
    adventure: Some(&SEEK_THE_BEAST),
    ..oracle::QUESTING_DRUID
};

pub static EXTRA: &[&CardDef] = &[&SEEK_THE_BEAST];

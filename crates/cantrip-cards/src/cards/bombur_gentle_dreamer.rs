use cantrip_api::*;
use cantrip_mechanics::storied;
use cantrip_oracle as oracle;

/// "Bombur doesn't untap during your untap step unless you have an enduring story" (CR 502.3, 702.195b): itself, while its
/// controller has no enduring story, read as its controller's untap step untaps permanents.
fn itself_without_an_enduring_story(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && !storied::you_have_an_enduring_story(q.state(), source)
}

fn doesnt_untap(_: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    chars.keywords = chars.keywords.with(Keyword::DoesNotUntap);
}

pub static CARD: CardDef = CardDef {
    storied: true,
    statics: &[static_ability(Layer::Ability, itself_without_an_enduring_story, doesnt_untap, "Bombur doesn't untap during your untap step unless you have an enduring story.")],
    ..oracle::BOMBUR_GENTLE_DREAMER
};

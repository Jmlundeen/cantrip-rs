using Cantrip.Server.Events;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// Every cube on the cube lists has a pick order beside it naming each of its cards once: a cube that changes is ranked
/// again (<c>python tools/maintenance/pick-order.py</c>) rather than left with cards the clock and the bots know nothing of.
/// </summary>
public sealed class CubePickOrderTests
{
    public static TheoryData<string> Cubes => [.. Directory.GetFiles(Path.Combine(RepoRoot.Path, "docs", "cube-lists"), "*.txt").Select(Path.GetFileName).OfType<string>()];

    [Theory]
    [MemberData(nameof(Cubes))]
    public void A_cubes_pick_order_ranks_every_card_in_it_once(string cube)
    {
        string path = Path.Combine(RepoRoot.Path, "docs", "cube-lists", cube);
        IReadOnlyList<string> order = CubeList.PickOrder(path);

        Assert.Equal(order.Count, order.Distinct(StringComparer.Ordinal).Count());
        Assert.Equal(CubeList.Load(path).Distinct(StringComparer.Ordinal).Order(StringComparer.Ordinal), order.Order(StringComparer.Ordinal));
    }

    [Fact]
    public void A_cube_without_a_pick_order_has_none()
    {
        Assert.Empty(CubeList.PickOrder(Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt")));
    }
}

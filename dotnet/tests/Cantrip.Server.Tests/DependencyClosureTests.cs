using System.Reflection;
using Cantrip.Client;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// What the client is allowed to link against, asserted rather than commented.
/// <para>
/// This test covers the client model. PresentationBoundaryTests additionally covers Cantrip.Client.Presentation,
/// which the web client draws from. The cards are server code — presentation must not
/// load one, let alone run one — and the mechanism that guarantees it is the reference graph, which is only a
/// guarantee for as long as something checks it. A stray <c>using</c> plus one convenient call is all it takes,
/// and the project file's comment will not stop it.
/// </para>
/// <para>
/// Read off the compiled assemblies, so it is the real closure: the compiler drops references nothing uses, so
/// what remains is what the code actually reached for.
/// </para>
/// </summary>
public sealed class DependencyClosureTests
{
    /// <summary>The only Cantrip assemblies the client's own logic is allowed to end up linked to.</summary>
    private static readonly string[] AllowedToTheClient = ["Cantrip.Client", "Cantrip.Core", "Cantrip.Protocol"];

    [Fact]
    public void The_client_reaches_protocol_and_core_and_nothing_else()
    {
        // A containment check rather than an equality one: the compiler drops a project reference the code never
        // used, so the exact list moves on its own. What must never move is the outer edge.
        Assert.Empty(CantripClosureOf(typeof(GameClient).Assembly).Except(AllowedToTheClient, StringComparer.Ordinal));
    }

    [Fact]
    public void The_wire_format_does_not_drag_the_server_along_with_it()
    {
        // The arrow that makes the rule structural: a client that speaks the protocol has not linked the server.
        Assert.Empty(
            CantripClosureOf(typeof(Cantrip.Protocol.TurnStepView).Assembly)
                .Except(["Cantrip.Core", "Cantrip.Protocol"], StringComparer.Ordinal));
    }

    [Fact]
    public void The_content_reader_is_reachable_only_from_the_server_side()
    {
        // The cards are read on the server; a client is sent them as the protocol's views.
        Assert.DoesNotContain("Cantrip.Datasets", CantripClosureOf(typeof(GameClient).Assembly));
        Assert.Contains("Cantrip.Datasets", CantripClosureOf(typeof(Cantrip.Server.ContentCache).Assembly));
    }

    /// <summary>Every <c>Cantrip.*</c> assembly reachable from one, including itself, sorted.</summary>
    private static IReadOnlyList<string> CantripClosureOf(Assembly root)
    {
        var seen = new SortedSet<string>(StringComparer.Ordinal);
        var queue = new Queue<Assembly>([root]);

        while (queue.Count > 0)
        {
            Assembly assembly = queue.Dequeue();
            string name = assembly.GetName().Name ?? string.Empty;

            if (!name.StartsWith("Cantrip.", StringComparison.Ordinal) || !seen.Add(name))
            {
                continue;
            }

            foreach (AssemblyName reference in assembly.GetReferencedAssemblies())
            {
                if (reference.Name is { } referenced && referenced.StartsWith("Cantrip.", StringComparison.Ordinal))
                {
                    queue.Enqueue(Assembly.Load(reference));
                }
            }
        }

        return [.. seen];
    }
}

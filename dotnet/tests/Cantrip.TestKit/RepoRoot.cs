namespace Cantrip.TestKit;

/// <summary>
/// Locates the repository root by walking up from the test assembly until the checkout turns up (the folder whose
/// dotnet folder holds the solution),
/// so tests read the real content files instead of each project copying content to its output
/// directory behind a brittle <c>../../../..</c> chain.
/// </summary>
public static class RepoRoot
{
    public static string Path { get; } = Find();

    /// <summary>The directory holding every dataset.</summary>
    public static string ContentRoot => System.IO.Path.Combine(Path, "content");

    public static string Content(string relative) =>
        System.IO.Path.Combine(Path, "content", relative.Replace('/', System.IO.Path.DirectorySeparatorChar));

    /// <summary>The decklists the demo table deals, one per seat in file-name order.</summary>
    public static string DemoDecks => System.IO.Path.Combine(Path, "decks", "demo");

    private static string Find()
    {
        var directory = new DirectoryInfo(AppContext.BaseDirectory);

        while (directory is not null)
        {
            if (File.Exists(System.IO.Path.Combine(directory.FullName, "dotnet", "Cantrip.sln")))
            {
                return directory.FullName;
            }

            directory = directory.Parent;
        }

        throw new InvalidOperationException($"Could not find dotnet/Cantrip.sln above {AppContext.BaseDirectory}.");
    }
}

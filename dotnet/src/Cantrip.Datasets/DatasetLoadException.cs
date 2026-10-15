namespace Cantrip.Datasets;

/// <summary>
/// A dataset could not be loaded.
/// <para>
/// Loading a dataset is all-or-nothing, like loading a card file: a dataset that half-loads is how a card ends up
/// silently doing nothing in a game, which reads as a rules bug and costs a day to find. There is no "skip the
/// broken card and carry on" path anywhere in this host.
/// </para>
/// </summary>
public class DatasetLoadException : Exception
{
    public DatasetLoadException(string folder, string message)
        : base($"{message}{Environment.NewLine}  (in {folder})") => Folder = folder;

    public DatasetLoadException(string folder, string message, Exception inner)
        : base($"{message}{Environment.NewLine}  (in {folder})", inner) => Folder = folder;

    public string Folder { get; }
}

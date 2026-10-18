using Cantrip.Art;

// Builds every rendition of every picture under the content folder into the art cache, so the cache can be copied to a
// host that has no pictures of its own (tools\publish-art.cmd). A rendition already built from the picture as it is now
// is left alone, so a second run takes seconds.
//
// Usage: cantrip-art-warm --content PATH --cache PATH

string? content = Arg("--content");
string? cache = Arg("--cache");
if (content is null || cache is null || !Directory.Exists(content))
{
    Console.Error.WriteLine("Usage: cantrip-art-warm --content PATH --cache PATH  (PATH is the folder holding the datasets)");
    return 2;
}

var store = new ArtStore(new ArtStoreOptions { ContentRoot = content, CacheDirectory = cache });

List<string> slugs =
[
    .. Directory.EnumerateDirectories(content)
        .Select(dataset => Path.Combine(dataset, "images"))
        .Where(Directory.Exists)
        .SelectMany(Directory.EnumerateFiles)
        .Select(Path.GetFileNameWithoutExtension)
        .OfType<string>()
        .Where(ArtStore.IsSlug)
        .Distinct(StringComparer.Ordinal)
        .Order(StringComparer.Ordinal),
];

int asked = 0;
int missed = 0;
int total = slugs.Count * ArtStore.Sizes.Count;
await Parallel.ForEachAsync(
    slugs.SelectMany(slug => ArtStore.Sizes.Select(size => (Slug: slug, Size: size))),
    new ParallelOptions { MaxDegreeOfParallelism = Math.Max(1, Environment.ProcessorCount / 2) },
    async (job, cancellation) =>
    {
        if (await store.GetAsync(job.Slug, job.Size, cancellation).ConfigureAwait(false) is null)
        {
            Interlocked.Increment(ref missed);
            Console.Error.WriteLine($"no rendition: {job.Slug} at {job.Size}");
        }

        if (Interlocked.Increment(ref asked) % 1000 == 0)
        {
            Console.WriteLine($"{asked} / {total}");
        }
    }).ConfigureAwait(false);

Console.WriteLine($"{slugs.Count} pictures, {asked} renditions, {missed} missing, in {cache}");
return missed == 0 ? 0 : 1;

string? Arg(string name)
{
    string[] args = Environment.GetCommandLineArgs();
    int at = Array.IndexOf(args, name);
    return at >= 0 && at + 1 < args.Length ? args[at + 1] : null;
}

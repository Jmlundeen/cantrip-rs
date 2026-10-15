namespace Cantrip.Client.Web;

/// <summary>
/// What drawing card faces costs, written to the browser's console when the page is opened with <c>?perf</c>: each
/// second in which faces were drawn, how many, how long they took, and how much the face cache holds; and once
/// the deck is ready, how long that took. Counted always - a few additions a face - and written only when asked for.
/// </summary>
internal static class FacePerf
{
    private static int _faces;
    private static int _durable;
    private static double _milliseconds;
    private static double _longest;
    private static DispatcherTimer? _timer;

    /// <summary>A face was drawn, for keeping or for the moment, in this long.</summary>
    public static void Drew(TimeSpan took, bool durable)
    {
        _faces++;
        _durable += durable ? 1 : 0;
        _milliseconds += took.TotalMilliseconds;
        _longest = Math.Max(_longest, took.TotalMilliseconds);

        if (ClientConfig.Perf && _timer is null)
        {
            _timer = new DispatcherTimer { Interval = TimeSpan.FromSeconds(1) };
            _timer.Tick += (_, _) => Write();
            _timer.Start();
        }
    }

    /// <summary>The deck was got ready: how many frames were decoded for it, in how long.</summary>
    public static void DeckReady(int frames, TimeSpan took)
    {
        if (ClientConfig.Perf)
        {
            Console.WriteLine($"[perf] deck ready: {frames} frames in {took.TotalMilliseconds:0} ms");
        }
    }

    private static void Write()
    {
        if (_faces == 0)
        {
            return;
        }

        Console.WriteLine(
            $"[perf] faces {_faces} ({_durable} durable) in {_milliseconds:0} ms, avg {_milliseconds / _faces:0.0} ms, longest {_longest:0.0} ms; " +
            $"kept {FaceCache.DurableBytes / 1024.0:0} KB durable, {FaceCache.MomentBytes / 1024.0:0} KB of the moment");
        _faces = 0;
        _durable = 0;
        _milliseconds = 0;
        _longest = 0;
    }
}

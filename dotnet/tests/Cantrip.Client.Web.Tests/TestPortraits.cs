using System.IO.Compression;
using System.Text;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// Players' pictures for a table to name, made here rather than served: each a small PNG in a <c>data:</c> URL, which
/// the client fetches as it would one beside the page. Each player's is their own colours - a disc on a ground - so the
/// pictures of a page tell apart whose is whose, and a player left out keeps the client's stand-in.
/// </summary>
internal static class TestPortraits
{
    /// <summary>Pictures for every player but those in <paramref name="without"/>.</summary>
    public static Func<string, string?> AllBut(params string[] without) =>
        player => without.Contains(player, StringComparer.Ordinal) ? null : For(player);

    public static string For(string player)
    {
        int hue = (int)((uint)player.Aggregate(17, (hash, c) => (hash * 31) + c) % 360);
        return "data:image/png;base64," + Convert.ToBase64String(Png(32, Hsv(hue, 0.55, 0.45), Hsv((hue + 40) % 360, 0.35, 0.95)));
    }

    private static (byte R, byte G, byte B) Hsv(int hue, double saturation, double value)
    {
        double c = value * saturation;
        double x = c * (1 - Math.Abs((hue / 60.0 % 2) - 1));
        (double r, double g, double b) = (hue / 60) switch
        {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        double m = value - c;
        return ((byte)((r + m) * 255), (byte)((g + m) * 255), (byte)((b + m) * 255));
    }

    /// <summary>A square PNG: <paramref name="disc"/> in the middle of <paramref name="ground"/>.</summary>
    private static byte[] Png(int side, (byte R, byte G, byte B) ground, (byte R, byte G, byte B) disc)
    {
        var rows = new MemoryStream();
        float middle = (side - 1) / 2f;
        for (int y = 0; y < side; y++)
        {
            rows.WriteByte(0);
            for (int x = 0; x < side; x++)
            {
                bool inside = ((x - middle) * (x - middle)) + ((y - middle) * (y - middle)) <= side * side / 9f;
                (byte r, byte g, byte b) = inside ? disc : ground;
                rows.Write([r, g, b]);
            }
        }

        var compressed = new MemoryStream();
        using (var zlib = new ZLibStream(compressed, CompressionLevel.Optimal, leaveOpen: true))
        {
            rows.Position = 0;
            rows.CopyTo(zlib);
        }

        var png = new MemoryStream();
        png.Write([0x89, (byte)'P', (byte)'N', (byte)'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        Chunk(png, "IHDR", [.. BigEndian(side), .. BigEndian(side), 8, 2, 0, 0, 0]);
        Chunk(png, "IDAT", compressed.ToArray());
        Chunk(png, "IEND", []);
        return png.ToArray();
    }

    private static void Chunk(Stream png, string type, byte[] data)
    {
        byte[] named = [.. Encoding.ASCII.GetBytes(type), .. data];
        png.Write(BigEndian(data.Length));
        png.Write(named);
        png.Write(BigEndian((int)Crc32(named)));
    }

    private static byte[] BigEndian(int value) => [(byte)(value >> 24), (byte)(value >> 16), (byte)(value >> 8), (byte)value];

    private static uint Crc32(byte[] bytes)
    {
        uint crc = 0xFFFFFFFF;
        foreach (byte b in bytes)
        {
            crc ^= b;
            for (int bit = 0; bit < 8; bit++) crc = (crc & 1) != 0 ? (crc >> 1) ^ 0xEDB88320 : crc >> 1;
        }

        return ~crc;
    }
}

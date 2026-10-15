using System.Globalization;
using System.Net;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;

namespace Cantrip.Client.Llm;

/// <summary>
/// A language model reached through OpenRouter's chat-completions endpoint.
/// <para>
/// OpenRouter rather than one provider's own SDK, and hand-written HTTP rather than a package: the whole request is
/// a model name, some messages and a temperature, which is the same shape at every provider that speaks this
/// endpoint. That keeps the experiment's real variable — which model is playing — a command-line argument instead of
/// a dependency, and keeps this repository's package list as short as it is.
/// </para>
/// <para>
/// Nothing here knows anything about Magic. What the model is asked is <see cref="Question"/>'s business and what it
/// is told is <see cref="GameText"/>'s; this only carries the conversation there and the reply back.
/// </para>
/// </summary>
public sealed class OpenRouterModel : ILanguageModel, IDisposable
{
    private const string Endpoint = "https://openrouter.ai/api/v1/chat/completions";

    /// <summary>The environment variable the key is read from when none is passed.</summary>
    public const string KeyVariable = "OPENROUTER_API_KEY";

    /// <summary>
    /// The model a seat plays with unless one is named.
    /// <para>
    /// Not the cheapest that works. A cheaper one plays legal Magic and visibly does not do combat arithmetic — its
    /// own stated reason for an attack comes back as "attack with your creatures that can deal damage" — so the
    /// question stopped being how far down this can go and became what it takes to get a real opponent. This is a
    /// default and not a decision: <c>--model</c> replaces it, and the cheap end is still worth re-testing whenever
    /// the questions change.
    /// </para>
    /// </summary>
    public const string DefaultModel = "deepseek/deepseek-v4-pro-0813";

    /// <summary>
    /// How many times one request is sent again before it is given up on. For the provider's own faults — a rate
    /// limit, a gateway error — not for a reply that named no choice, which <see cref="ModelChooser"/> handles by
    /// asking a better question.
    /// </summary>
    private const int Attempts = 4;

    private readonly HttpClient _http;
    private readonly string _model;
    private readonly double _temperature;
    private readonly int _maxTokens;

    /// <param name="model">The OpenRouter model id, such as <c>deepseek/deepseek-v4-pro-0813</c>.</param>
    /// <param name="key">The API key, or null to read <see cref="KeyVariable"/> from the environment.</param>
    /// <param name="temperature">
    /// Low by default. This is not creative writing: the same board twice should mostly get the same answer, so that
    /// a game that goes badly says something about the model rather than about the draw.
    /// </param>
    /// <param name="maxTokens">Room for the reply. A key is a token; the rest is whatever reasoning the model shows.</param>
    /// <param name="timeout">How long one request may take.</param>
    public OpenRouterModel(
        string? model = null,
        string? key = null,
        double temperature = 0.2,
        int maxTokens = 2048,
        TimeSpan? timeout = null)
    {
        string secret = key ?? Environment.GetEnvironmentVariable(KeyVariable) ?? string.Empty;

        if (secret.Length == 0)
        {
            throw new InvalidOperationException(
                $"No OpenRouter key. Set {KeyVariable} in the environment, or pass --key. "
                + "A key is made at https://openrouter.ai/keys.");
        }

        _model = model ?? DefaultModel;
        _temperature = temperature;
        _maxTokens = maxTokens;

        _http = new HttpClient { Timeout = timeout ?? TimeSpan.FromMinutes(2) };
        _http.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", secret);

        // OpenRouter attributes traffic by these two, and they cost nothing to send. Named so a key's usage page
        // says which experiment spent the credits.
        _http.DefaultRequestHeaders.Add("HTTP-Referer", "https://github.com/cantrip");
        _http.DefaultRequestHeaders.Add("X-Title", "Cantrip");
    }

    /// <summary>The model this seat is playing with, for a run to report.</summary>
    public string Model => _model;

    public ModelUsage Usage { get; } = new();

    public string Continue(IReadOnlyList<ChatTurn> conversation)
    {
        ArgumentNullException.ThrowIfNull(conversation);

        // Run on the thread pool and wait, rather than awaited: the seat is called from a loop that wants an answer
        // or nothing (see ILanguageModel), and Task.Run keeps a caller with a synchronisation context of its own — a
        // UI thread — from deadlocking on its own thread.
        return Task.Run(() => Send(conversation)).GetAwaiter().GetResult();
    }

    /// <summary>
    /// What this model charges per million tokens, in and out, or null where it could not be found out.
    /// <para>
    /// Read off the service rather than kept in a table here: prices change, there are hundreds of models, and a
    /// table in this repository would be wrong by the time anybody read it. A run that cannot reach the list simply
    /// reports tokens and no dollars.
    /// </para>
    /// </summary>
    public (decimal In, decimal Out)? Prices()
    {
        try
        {
            return Task.Run(Lookup).GetAwaiter().GetResult();
        }
        catch (Exception error) when (error is HttpRequestException or TaskCanceledException or JsonException)
        {
            return null;
        }
    }

    private async Task<(decimal In, decimal Out)?> Lookup()
    {
        using HttpResponseMessage response = await _http
            .GetAsync(new Uri("https://openrouter.ai/api/v1/models"))
            .ConfigureAwait(false);

        if (!response.IsSuccessStatusCode)
        {
            return null;
        }

        using JsonDocument document = JsonDocument.Parse(
            await response.Content.ReadAsStringAsync().ConfigureAwait(false));

        foreach (JsonElement model in document.RootElement.GetProperty("data").EnumerateArray())
        {
            if (!string.Equals(model.GetProperty("id").GetString(), _model, StringComparison.Ordinal)
                || !model.TryGetProperty("pricing", out JsonElement pricing))
            {
                continue;
            }

            // Priced per token as a string, which is the only place a price with eight decimal places survives a JSON
            // number intact.
            return (Price(pricing, "prompt") * 1_000_000m, Price(pricing, "completion") * 1_000_000m);
        }

        return null;
    }

    private static decimal Price(JsonElement pricing, string name) =>
        pricing.TryGetProperty(name, out JsonElement value)
        && decimal.TryParse(value.GetString(), NumberStyles.Float, CultureInfo.InvariantCulture, out decimal price)
            ? price
            : 0m;

    public void Dispose() => _http.Dispose();

    private async Task<string> Send(IReadOnlyList<ChatTurn> conversation)
    {
        string body = Request(conversation);
        Exception? last = null;

        for (int attempt = 1; attempt <= Attempts; attempt++)
        {
            try
            {
                using var content = new StringContent(body, Encoding.UTF8, "application/json");
                using HttpResponseMessage response = await _http.PostAsync(new Uri(Endpoint), content)
                    .ConfigureAwait(false);

                string text = await response.Content.ReadAsStringAsync().ConfigureAwait(false);

                if (response.IsSuccessStatusCode)
                {
                    return Reply(text);
                }

                var failure = new HttpRequestException(
                    $"OpenRouter answered {(int)response.StatusCode} {response.StatusCode}: {Trim(text)}");

                if (!Worth(response.StatusCode))
                {
                    throw failure;
                }

                last = failure;
            }
            catch (Exception error) when (error is HttpRequestException or TaskCanceledException && attempt < Attempts)
            {
                last = error;
            }

            if (attempt < Attempts)
            {
                Usage.Retried();
                await Task.Delay(TimeSpan.FromSeconds(Math.Pow(2, attempt))).ConfigureAwait(false);
            }
        }

        throw new HttpRequestException(
            $"OpenRouter did not answer after {Attempts} attempts: {last?.Message}", last);
    }

    /// <summary>Whether a refusal is one worth sending again: a rate limit or the provider's own fault, never ours.</summary>
    private static bool Worth(HttpStatusCode status) =>
        status == HttpStatusCode.TooManyRequests || (int)status >= 500;

    private string Request(IReadOnlyList<ChatTurn> conversation)
    {
        var buffer = new MemoryStream();

        using (var json = new Utf8JsonWriter(buffer))
        {
            json.WriteStartObject();
            json.WriteString("model", _model);
            json.WriteNumber("temperature", _temperature);
            json.WriteNumber("max_tokens", _maxTokens);

            json.WriteStartArray("messages");

            foreach (ChatTurn turn in conversation)
            {
                json.WriteStartObject();
                json.WriteString("role", Role(turn.Role));
                json.WriteString("content", turn.Text);
                json.WriteEndObject();
            }

            json.WriteEndArray();
            json.WriteEndObject();
        }

        return Encoding.UTF8.GetString(buffer.ToArray());
    }

    private static string Role(ChatRole role) => role switch
    {
        ChatRole.System => "system",
        ChatRole.Assistant => "assistant",
        _ => "user",
    };

    /// <summary>
    /// The message out of a successful body, and the tokens it cost.
    /// <para>
    /// A 200 is not proof of an answer: OpenRouter reports a provider that fell over in the body, and a model that
    /// spent its whole allowance thinking returns an empty message. Both are told apart here rather than handed on as
    /// an answer of no letters, which the caller could only read as a refusal to choose.
    /// </para>
    /// </summary>
    private string Reply(string body)
    {
        using JsonDocument document = JsonDocument.Parse(body);
        JsonElement root = document.RootElement;

        if (root.TryGetProperty("error", out JsonElement error))
        {
            throw new HttpRequestException($"OpenRouter answered with an error: {Trim(error.ToString())}");
        }

        if (root.TryGetProperty("usage", out JsonElement usage))
        {
            Usage.Record(Count(usage, "prompt_tokens"), Count(usage, "completion_tokens"));
        }

        if (!root.TryGetProperty("choices", out JsonElement choices)
            || choices.ValueKind != JsonValueKind.Array
            || choices.GetArrayLength() == 0)
        {
            throw new HttpRequestException($"OpenRouter answered with no choices: {Trim(body)}");
        }

        JsonElement message = choices[0].GetProperty("message");

        string? text = message.TryGetProperty("content", out JsonElement content)
            ? content.GetString()
            : null;

        // Some models put everything in a reasoning field and leave content empty. The key is in there somewhere,
        // and a tolerant reader beats refusing to play with half the models on the service.
        if (string.IsNullOrWhiteSpace(text) && message.TryGetProperty("reasoning", out JsonElement reasoning))
        {
            text = reasoning.GetString();
        }

        return text ?? string.Empty;
    }

    private static long Count(JsonElement usage, string name) =>
        usage.TryGetProperty(name, out JsonElement value) && value.TryGetInt64(out long tokens) ? tokens : 0;

    /// <summary>A failure body cut to something that fits in a console line.</summary>
    private static string Trim(string text)
    {
        string one = text.ReplaceLineEndings(" ").Trim();
        return one.Length <= 400 ? one : string.Concat(one.AsSpan(0, 400), "...");
    }

    /// <summary>The model id and what it has cost, for a run's last line.</summary>
    public override string ToString() =>
        string.Create(CultureInfo.InvariantCulture, $"{_model}: {Usage}");
}

using System.Reflection;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Serialization;
using System.Text.Json.Serialization.Metadata;
using Cantrip.Protocol;
using MessagePack;

namespace Cantrip.Server.Rules;

/// <summary>
/// <c>Cantrip.Protocol</c> as JSON, the way the native host speaks it (<c>cantrip-host</c>): PascalCase
/// members as the C# properties are named, enums as their member names, <see cref="ViewHandle"/> as a bare integer,
/// and every union — prompts, answers, events — tagged <c>$type</c> with the C# class name.
/// <para>
/// The protocol's own serialiser is MessagePack, keyed by number, which a Rust process could speak but nobody could
/// read in a log. JSON lines are what the sidecar's stdin and stdout carry, so this is the one place the protocol's
/// shapes are written out by name, and the shapes are the same ones the MessagePack unions declare: a union member is
/// found by the <see cref="UnionAttribute"/>s already on the base class, so a prompt added to the wire is JSON-ready
/// the moment it is in its union.
/// </para>
/// </summary>
public static class ProtocolJson
{
    /// <summary>The settings every request and reply is read and written with.</summary>
    public static JsonSerializerOptions Options { get; } = Create();

    public static string Serialize<T>(T value) => JsonSerializer.Serialize(value, Options);

    public static T? Deserialize<T>(string json) => JsonSerializer.Deserialize<T>(json, Options);

    public static T? Deserialize<T>(JsonElement element) => element.Deserialize<T>(Options);

    private static JsonSerializerOptions Create()
    {
        var options = new JsonSerializerOptions
        {
            // Property names as declared, which is PascalCase throughout the protocol.
            PropertyNamingPolicy = null,
            DefaultIgnoreCondition = JsonIgnoreCondition.Never,
            TypeInfoResolver = new DefaultJsonTypeInfoResolver { Modifiers = { OnlyWireMembers } },
        };

        options.Converters.Add(new ViewHandleConverter());
        options.Converters.Add(new EnumNameConverterFactory());
        options.Converters.Add(new UnionConverter<PromptPayload>());
        options.Converters.Add(new UnionConverter<AnswerPayload>());
        options.Converters.Add(new UnionConverter<GameEventView>());
        return options;
    }

    /// <summary>
    /// A protocol type's computed members — a prompt's <c>Preferred</c>, a target's <c>IsPlayer</c> — are not on the
    /// wire: MessagePack ignores them, and so does this. Only the protocol's: a request written as an anonymous object
    /// has nothing but read-only members, and all of them are the request.
    /// </summary>
    private static void OnlyWireMembers(JsonTypeInfo info)
    {
        if (info.Kind != JsonTypeInfoKind.Object || info.Type.Namespace != typeof(PlayerView).Namespace)
        {
            return;
        }

        for (int index = info.Properties.Count - 1; index >= 0; index--)
        {
            if (info.Properties[index].Set is null)
            {
                info.Properties.RemoveAt(index);
            }
        }
    }

    /// <summary>A handle is its number and nothing more; zero is none.</summary>
    private sealed class ViewHandleConverter : JsonConverter<ViewHandle>
    {
        public override ViewHandle Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options) =>
            reader.TokenType switch
            {
                JsonTokenType.Number => new ViewHandle(reader.GetInt32()),
                JsonTokenType.Null => ViewHandle.None,
                _ => throw new JsonException($"A ViewHandle is a bare integer, not {reader.TokenType}."),
            };

        public override void Write(Utf8JsonWriter writer, ViewHandle value, JsonSerializerOptions options) =>
            writer.WriteNumberValue(value.Value);
    }

    /// <summary>
    /// Enums by member name. Tolerant of a number, which is the MessagePack form, and of null, which the Rust mirror
    /// sends for an option it has none of (a view's step before the first turn), read as the enum's zero.
    /// </summary>
    private sealed class EnumNameConverterFactory : JsonConverterFactory
    {
        public override bool CanConvert(Type typeToConvert) => typeToConvert.IsEnum;

        public override JsonConverter? CreateConverter(Type typeToConvert, JsonSerializerOptions options) =>
            (JsonConverter?)Activator.CreateInstance(typeof(EnumNameConverter<>).MakeGenericType(typeToConvert));
    }

    private sealed class EnumNameConverter<T> : JsonConverter<T>
        where T : struct, Enum
    {
        public override bool HandleNull => true;

        public override T Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            switch (reader.TokenType)
            {
                case JsonTokenType.Null:
                    return default;

                case JsonTokenType.Number:
                    return (T)Enum.ToObject(typeof(T), reader.GetInt64());

                case JsonTokenType.String:
                    string name = reader.GetString()!;
                    return Enum.TryParse(name, ignoreCase: false, out T value) && Enum.IsDefined(value)
                        ? value
                        : throw new JsonException($"\"{name}\" is not a member of {typeof(T).Name}.");

                default:
                    throw new JsonException($"A {typeof(T).Name} is a member name, not {reader.TokenType}.");
            }
        }

        public override void Write(Utf8JsonWriter writer, T value, JsonSerializerOptions options) =>
            writer.WriteStringValue(value.ToString());
    }

    /// <summary>
    /// A union member tagged with its class name: <c>{"$type":"MayPromptMessage","Player":1,...}</c>. The members are
    /// the base class's <see cref="UnionAttribute"/>s, so the JSON vocabulary and the MessagePack one cannot drift.
    /// </summary>
    private sealed class UnionConverter<TBase> : JsonConverter<TBase>
        where TBase : class
    {
        private const string Tag = "$type";

        private static readonly Dictionary<string, Type> Members = typeof(TBase)
            .GetCustomAttributes<UnionAttribute>()
            .ToDictionary(union => union.SubType.Name, union => union.SubType, StringComparer.Ordinal);

        // Only the base type: a concrete member is read and written as the plain object it is, with this converter
        // out of the way, which is what lets Read hand the element to the ordinary deserialiser.
        public override bool CanConvert(Type typeToConvert) => typeToConvert == typeof(TBase);

        public override TBase? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
        {
            if (reader.TokenType == JsonTokenType.Null)
            {
                return null;
            }

            using JsonDocument document = JsonDocument.ParseValue(ref reader);
            JsonElement root = document.RootElement;

            if (root.ValueKind != JsonValueKind.Object || !root.TryGetProperty(Tag, out JsonElement tag))
            {
                throw new JsonException($"A {typeof(TBase).Name} is an object with a \"{Tag}\" naming its kind.");
            }

            string name = tag.GetString() ?? string.Empty;
            if (!Members.TryGetValue(name, out Type? member))
            {
                throw new JsonException($"\"{name}\" is not a {typeof(TBase).Name}: it has no [Union] key on it.");
            }

            return (TBase?)root.Deserialize(member, options);
        }

        public override void Write(Utf8JsonWriter writer, TBase value, JsonSerializerOptions options)
        {
            Type member = value.GetType();
            if (!Members.ContainsKey(member.Name))
            {
                throw new JsonException($"{member.Name} has no [Union] key on {typeof(TBase).Name}, so it cannot be sent.");
            }

            // The tag first, then the members as the plain object serialises them.
            JsonObject body = JsonSerializer.SerializeToNode(value, member, options)!.AsObject();
            writer.WriteStartObject();
            writer.WriteString(Tag, member.Name);
            foreach ((string name, JsonNode? child) in body)
            {
                writer.WritePropertyName(name);
                if (child is null)
                {
                    writer.WriteNullValue();
                }
                else
                {
                    child.WriteTo(writer, options);
                }
            }

            writer.WriteEndObject();
        }
    }
}

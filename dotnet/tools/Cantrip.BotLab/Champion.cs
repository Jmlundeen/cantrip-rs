using System.Collections;
using System.Collections.Concurrent;
using System.Diagnostics;
using System.Globalization;
using System.Reflection;
using System.Runtime.Loader;
using Cantrip.Client;
using Cantrip.Protocol;
using MessagePack;

namespace Cantrip.BotLab;

/// <summary>
/// A frozen build of an earlier tactical seat, kept so that today's seat can be played against it.
/// <para>
/// Against the random bot every version of the pilot wins nine games in ten, which says nothing about a change to
/// it. What does is the pilot before the change, in the other seat, on the same decks. A champion is that: the
/// compiled client of some earlier commit, copied into the lab's folder (<c>.cantrip-lab\champions\NAME</c>) and
/// loaded here beside the current one. It is real earlier code, weights and all, not a setting of the current code.
/// </para>
/// <para>
/// The two builds never share a type. The frozen one is loaded into a load context of its own, with its own copy of
/// the protocol, and plays with its own <c>GameClient</c> and its own <c>TacticalPlay</c>; what crosses between the
/// two is what would cross a socket, the protocol's messages as bytes. So a champion goes on working after the
/// current client's types have changed under it, for as long as the wire format it was built against is still
/// spoken, which is the same promise an old client in a browser tab is given.
/// </para>
/// </summary>
public sealed class Champion
{
    private const string LabFolder = ".cantrip-lab";

    private static readonly ConcurrentDictionary<string, Champion> Loaded = new(StringComparer.OrdinalIgnoreCase);

    private readonly Type _transport;
    private readonly Type _client;
    private readonly Type _bot;
    private readonly Type _serverMessage;
    private readonly MethodInfo _serialize;
    private readonly MethodInfo _deserialize;
    private readonly MethodInfo _pump;
    private readonly MethodInfo _act;
    private readonly PropertyInfo _retrying;

    private Champion(string name, string folder)
    {
        Name = name;
        Folder = folder;

        var context = new Frozen(folder);
        Assembly client = context.LoadFromAssemblyPath(Path.Combine(folder, "Cantrip.Client.dll"));
        Assembly protocol = context.LoadFromAssemblyPath(Path.Combine(folder, "Cantrip.Protocol.dll"));
        Assembly messagePack = context.LoadFromAssemblyPath(Path.Combine(folder, "MessagePack.dll"));

        Type Need(Assembly assembly, string type) =>
            assembly.GetType(type) ?? throw new InvalidOperationException($"Champion '{name}' has no {type}: not a build this lab can seat.");

        _transport = Need(protocol, "Cantrip.Protocol.IGameTransport");
        _serverMessage = Need(protocol, "Cantrip.Protocol.ServerMessage");
        Type clientMessage = Need(protocol, "Cantrip.Protocol.ClientMessage");
        _client = Need(client, "Cantrip.Client.GameClient");
        _bot = Need(client, "Cantrip.Client.Tactics.TacticalPlay");

        // Its own serializer, over its own message types: MessagePackSerializer.Serialize<T>(T, options, token) and
        // Deserialize<T>(ReadOnlyMemory<byte>, options, token).
        Type serializer = Need(messagePack, "MessagePack.MessagePackSerializer");
        _serialize = serializer.GetMethods(BindingFlags.Public | BindingFlags.Static)
            .First(method => method is { Name: "Serialize", IsGenericMethodDefinition: true } && method.ReturnType == typeof(byte[])
                && method.GetParameters() is [{ ParameterType.IsGenericParameter: true }, _, _])
            .MakeGenericMethod(clientMessage);
        _deserialize = serializer.GetMethods(BindingFlags.Public | BindingFlags.Static)
            .First(method => method is { Name: "Deserialize", IsGenericMethodDefinition: true }
                && method.GetParameters() is [{ } buffer, _, _] && buffer.ParameterType == typeof(ReadOnlyMemory<byte>))
            .MakeGenericMethod(_serverMessage);

        _pump = _client.GetMethod("Pump") ?? throw new InvalidOperationException($"Champion '{name}': GameClient has no Pump.");
        _act = _bot.GetMethod("Act") ?? throw new InvalidOperationException($"Champion '{name}': TacticalPlay has no Act.");
        _retrying = _bot.GetProperty("Retrying") ?? throw new InvalidOperationException($"Champion '{name}': TacticalPlay has no Retrying.");
    }

    public string Name { get; }

    public string Folder { get; }

    // --- Where champions are kept ------------------------------------------------------------------------------------

    /// <summary>
    /// The lab's own folder: <c>CANTRIP_LAB</c> if set, else the nearest <c>.cantrip-lab</c> at or above the
    /// repository root — a worktree under the main checkout finds the main checkout's, so every worktree seats the
    /// same champions — else one made at the root. Nothing in it is in git.
    /// </summary>
    public static string Lab(string root)
    {
        if (Environment.GetEnvironmentVariable("CANTRIP_LAB") is { Length: > 0 } named)
        {
            return named;
        }

        for (DirectoryInfo? here = new(root); here is not null; here = here.Parent)
        {
            if (Directory.Exists(Path.Combine(here.FullName, LabFolder)))
            {
                return Path.Combine(here.FullName, LabFolder);
            }
        }

        return Path.Combine(root, LabFolder);
    }

    private static string Champions(string root) => Path.Combine(Lab(root), "champions");

    /// <summary>
    /// The champion a seat name means, or null for a name that is not one: <c>champion</c> is whichever is current,
    /// <c>referee</c> the one pinned for the draft lab's numbers, <c>champion:NAME</c> that one.
    /// </summary>
    public static Champion? Named(string root, string seat)
    {
        ArgumentNullException.ThrowIfNull(seat);

        string? name = seat switch
        {
            "champion" => Pointer(root, "current"),
            "referee" => Pointer(root, "referee"),
            _ when seat.StartsWith("champion:", StringComparison.Ordinal) => seat["champion:".Length..],
            _ => null,
        };

        if (name is null)
        {
            return null;
        }

        string folder = Path.Combine(Champions(root), name);
        if (!File.Exists(Path.Combine(folder, "Cantrip.Client.dll")))
        {
            throw new FileNotFoundException($"No champion '{name}' in {Champions(root)}. Freeze one: cantrip-botlab champion freeze NAME.");
        }

        return Loaded.GetOrAdd(folder, _ => new Champion(name, folder));
    }

    private static string Pointer(string root, string which)
    {
        string file = Path.Combine(Champions(root), which + ".txt");
        return File.Exists(file) && File.ReadAllText(file).Trim() is { Length: > 0 } name
            ? name
            : throw new FileNotFoundException($"No {which} champion is named in {file}. Name one: cantrip-botlab champion {(which == "current" ? "use" : which)} NAME.");
    }

    /// <summary>A seat played by this champion: one per seat per game, as any other way of playing.</summary>
    public IAutoPlayer Seat() => new Player(this);

    // --- The command -------------------------------------------------------------------------------------------------

    /// <summary><c>champion freeze NAME</c>, <c>champion use NAME</c>, <c>champion referee NAME</c>, <c>champion list</c>.</summary>
    public static int Run(string root, string[] args)
    {
        ArgumentNullException.ThrowIfNull(args);

        string folder = Champions(root);
        string verb = args.ElementAtOrDefault(1) ?? "list";
        string name = args.ElementAtOrDefault(2) ?? string.Empty;

        switch (verb)
        {
            case "freeze" when name.Length > 0:
                return Freeze(root, args, name);

            case "use" or "referee" when name.Length > 0:
                if (!File.Exists(Path.Combine(folder, name, "Cantrip.Client.dll")))
                {
                    Console.Error.WriteLine($"No champion '{name}' in {folder}.");
                    return 2;
                }

                File.WriteAllText(Path.Combine(folder, (verb == "use" ? "current" : "referee") + ".txt"), name + "\n");
                Console.WriteLine(verb == "use"
                    ? $"The seat named 'champion' is now {name}."
                    : $"The seat named 'referee' is now {name}.");
                return 0;

            case "list":
                if (!Directory.Exists(folder))
                {
                    Console.WriteLine($"No champions yet in {folder}.");
                    return 0;
                }

                string current = File.Exists(Path.Combine(folder, "current.txt")) ? File.ReadAllText(Path.Combine(folder, "current.txt")).Trim() : string.Empty;
                string referee = File.Exists(Path.Combine(folder, "referee.txt")) ? File.ReadAllText(Path.Combine(folder, "referee.txt")).Trim() : string.Empty;

                Console.WriteLine($"Champions in {folder}:");
                foreach (string each in Directory.GetDirectories(folder).Order(StringComparer.Ordinal))
                {
                    string id = Path.GetFileName(each);
                    string about = File.Exists(Path.Combine(each, "champion.txt")) ? File.ReadLines(Path.Combine(each, "champion.txt")).FirstOrDefault() ?? string.Empty : string.Empty;
                    string marks = (id == current ? " [champion]" : string.Empty) + (id == referee ? " [referee]" : string.Empty);
                    Console.WriteLine($"  champion:{id}{marks}  {about}");
                }

                return 0;

            default:
                Console.Error.WriteLine("champion freeze NAME [--from DIR] [--commit ID] [--note TEXT] | champion use NAME | champion referee NAME | champion list");
                return 2;
        }
    }

    /// <summary>
    /// Copies a built client into the lab's folder under a name: the client, the protocol it speaks and whatever
    /// else of the folder those two refer to, and a note of what it was built from. From this lab's own build unless
    /// <c>--from</c> names another folder with one in it.
    /// </summary>
    private static int Freeze(string root, string[] args, string name)
    {
        string from = Value(args, "--from") ?? AppContext.BaseDirectory;
        string into = Path.Combine(Champions(root), name);

        if (!File.Exists(Path.Combine(from, "Cantrip.Client.dll")))
        {
            Console.Error.WriteLine($"No Cantrip.Client.dll in {from}.");
            return 2;
        }

        if (Directory.Exists(into))
        {
            Console.Error.WriteLine($"There is already a champion '{name}': a frozen build is never written over. Give this one another name.");
            return 2;
        }

        // What the client refers to, and what those refer to, as far as the folder holds them: the rest is the runtime's.
        HashSet<string> wanted = new(StringComparer.OrdinalIgnoreCase);
        Queue<string> queue = new(["Cantrip.Client", "Cantrip.Protocol", "MessagePack"]);
        var probe = new AssemblyLoadContext("freeze", isCollectible: true);

        while (queue.TryDequeue(out string? next))
        {
            string file = Path.Combine(from, next + ".dll");
            if (!File.Exists(file) || !wanted.Add(next))
            {
                continue;
            }

            using FileStream stream = File.OpenRead(file);
            foreach (AssemblyName reference in probe.LoadFromStream(stream).GetReferencedAssemblies())
            {
                if (reference.Name is { } referenced)
                {
                    queue.Enqueue(referenced);
                }
            }
        }

        probe.Unload();

        Directory.CreateDirectory(into);
        foreach (string assembly in wanted)
        {
            File.Copy(Path.Combine(from, assembly + ".dll"), Path.Combine(into, assembly + ".dll"));
        }

        string commit = Value(args, "--commit") ?? Git(root, "rev-parse --short=8 HEAD");
        string dirty = Value(args, "--commit") is null && Git(root, "status --porcelain -- dotnet/src/Cantrip.Client dotnet/src/Cantrip.Protocol").Length > 0
            ? " with uncommitted changes to the client" : string.Empty;
        string note = Value(args, "--note") ?? string.Empty;

        File.WriteAllText(
            Path.Combine(into, "champion.txt"),
            $"The tactical seat as of {commit}{dirty}, frozen {DateTime.Now.ToString("yyyy-MM-dd HH:mm", CultureInfo.InvariantCulture)}. {note}".TrimEnd() + "\n" +
            $"Built in {from}.\n" +
            $"Assemblies: {string.Join(", ", wanted.Order(StringComparer.OrdinalIgnoreCase))}.\n" +
            $"Seat it as champion:{name} (cantrip-botlab duel --a tactical --b champion:{name}).\n");

        Console.WriteLine($"Frozen as champion:{name} in {into} ({wanted.Count} assemblies, the client of {commit}{dirty}).");

        // The first champion there is, is the champion.
        if (!File.Exists(Path.Combine(Champions(root), "current.txt")))
        {
            File.WriteAllText(Path.Combine(Champions(root), "current.txt"), name + "\n");
            Console.WriteLine($"The seat named 'champion' is now {name}.");
        }

        return 0;
    }

    private static string? Value(string[] args, string name)
    {
        int at = Array.IndexOf(args, name);
        return at >= 0 && at + 1 < args.Length ? args[at + 1] : null;
    }

    private static string Git(string root, string arguments)
    {
        try
        {
            using var git = Process.Start(new ProcessStartInfo("git", $"-C \"{root}\" {arguments}")
            {
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
            });

            string said = git?.StandardOutput.ReadToEnd().Trim() ?? string.Empty;
            git?.WaitForExit();
            return said;
        }
        catch (Exception error) when (error is InvalidOperationException or System.ComponentModel.Win32Exception)
        {
            return "an unknown commit";
        }
    }

    // --- A frozen build, loaded --------------------------------------------------------------------------------------

    /// <summary>
    /// A load context that finds an assembly in the champion's folder before anywhere else. Without it the frozen
    /// client would be handed the current protocol, which is already loaded, and the two builds would share types.
    /// </summary>
    private sealed class Frozen(string folder) : AssemblyLoadContext("champion " + Path.GetFileName(folder))
    {
        protected override Assembly? Load(AssemblyName assemblyName)
        {
            string file = Path.Combine(folder, assemblyName.Name + ".dll");
            return File.Exists(file) ? LoadFromAssemblyPath(file) : null;
        }
    }

    /// <summary>
    /// The seat. To the lab it is a way of playing like any other; inside, the frozen build's own client is shown
    /// every message this seat's client is, and whatever it answers is sent as this seat's answer.
    /// </summary>
    private sealed class Player : IAutoPlayer, ISeatTransport
    {
        private readonly Champion _champion;
        private readonly Wire _wire;
        private readonly object _client;
        private readonly object _bot;
        private Tee? _tee;

        public Player(Champion champion)
        {
            _champion = champion;

            _wire = (Wire)DispatchProxy.Create(champion._transport, typeof(Wire));
            _wire.Bind(champion);

            _client = Activator.CreateInstance(champion._client, _wire)!;

            // Whatever its constructor takes, it is given nothing: the weights it shipped with, and no seed.
            ConstructorInfo constructor = champion._bot.GetConstructors().OrderBy(each => each.GetParameters().Length).First();
            _bot = constructor.Invoke([.. constructor.GetParameters().Select(parameter => parameter.HasDefaultValue ? parameter.DefaultValue : null)]);
        }

        public bool Retrying => (bool)_champion._retrying.GetValue(_bot)!;

        public IGameTransport Wrap(IGameTransport transport) => _tee = new Tee(transport);

        public bool Act(GameClient client)
        {
            ArgumentNullException.ThrowIfNull(client);

            if (_tee is null)
            {
                throw new InvalidOperationException("A champion's seat has to be given its transport before the game starts (ISeatTransport).");
            }

            try
            {
                foreach (ServerMessage message in _tee.Take())
                {
                    _wire.Inbound.Add(_champion._deserialize.Invoke(null, [(ReadOnlyMemory<byte>)MessagePackSerializer.Serialize(message), null, default(CancellationToken)]));
                }

                _champion._pump.Invoke(_client, null);

                _wire.Outbound = client;
                return (bool)_champion._act.Invoke(_bot, [_client])!;
            }
            catch (TargetInvocationException error) when (error.InnerException is { } inner)
            {
                throw new InvalidOperationException($"champion:{_champion.Name}: {inner.GetType().Name}: {inner.Message}", inner);
            }
        }
    }

    /// <summary>The seat's own transport, remembering everything that came in so the frozen client can be shown it too.</summary>
    private sealed class Tee(IGameTransport inner) : IGameTransport
    {
        private readonly List<ServerMessage> _seen = [];

        public bool IsConnected => inner.IsConnected;

        public void Send(ClientMessage message) => inner.Send(message);

        public IReadOnlyList<ServerMessage> Receive()
        {
            IReadOnlyList<ServerMessage> received = inner.Receive();
            _seen.AddRange(received);
            return received;
        }

        public List<ServerMessage> Take()
        {
            List<ServerMessage> seen = [.. _seen];
            _seen.Clear();
            return seen;
        }

        public void Dispose() => inner.Dispose();
    }

    /// <summary>
    /// The frozen client's transport, which is an interface of the frozen protocol and so cannot be implemented in
    /// this build's code: a proxy made at run time stands in for it. What the frozen client receives is what this
    /// seat was sent; what it sends, when it is an answer, is given as this seat's client's answer.
    /// </summary>
    public class Wire : DispatchProxy
    {
        internal Champion Champion { get; private set; } = null!;

        internal IList Inbound { get; private set; } = null!;

        internal void Bind(Champion champion)
        {
            Champion = champion;
            Inbound = Fresh();
        }

        internal GameClient? Outbound { get; set; }

        protected override object? Invoke(MethodInfo? targetMethod, object?[]? args)
        {
            switch (targetMethod?.Name)
            {
                case "get_IsConnected":
                    return true;

                case "Receive":
                    IList received = Inbound;
                    Inbound = Fresh();
                    return received;

                case "Send" when args is [{ } message]:
                    byte[] bytes = (byte[])Champion._serialize.Invoke(null, [message, null, default(CancellationToken)])!;

                    // Only an answer, and a check of the choice it is making, are this seat's to send: the lab has already joined
                    // the table for it.
                    ClientMessage sent = MessagePackSerializer.Deserialize<ClientMessage>(bytes);
                    if (sent is AnswerMessage answer && Outbound is { IsWaitingOnMe: true } client)
                    {
                        client.Answer(answer.Answer);
                    }
                    else if (sent is CheckSelectionMessage check && Outbound is { } checking)
                    {
                        checking.Check(check.Selected);
                    }
                    else if (sent is CheckBlocksMessage blocks && Outbound is { } blocking)
                    {
                        // A declaration of blockers or of attackers is settled with the host before it is answered: a champion
                        // whose check went nowhere would wait on a verdict for ever.
                        blocking.CheckBlocks(blocks.Blocks);
                    }
                    else if (sent is CheckAttacksMessage attacks && Outbound is { } attacking)
                    {
                        attacking.CheckAttacks(attacks.Attacks);
                    }

                    return null;

                default:
                    return null;
            }
        }

        private IList Fresh() => (IList)Activator.CreateInstance(typeof(List<>).MakeGenericType(Champion._serverMessage))!;
    }
}

/// <summary>
/// A way of playing that wants to see its seat's transport before the client is made over it. <see cref="Match"/>
/// asks every seat's player; a champion answers with a transport that also remembers what was received.
/// </summary>
public interface ISeatTransport
{
    IGameTransport Wrap(IGameTransport transport);
}

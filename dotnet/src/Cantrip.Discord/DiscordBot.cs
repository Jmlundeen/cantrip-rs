using System.Collections.Concurrent;
using Cantrip.Server.Events;
using Discord;
using Discord.Net;
using Discord.WebSocket;

namespace Cantrip.Discord;

public sealed class DiscordBotOptions
{
    /// <summary>The bot token. A secret: from the environment, never the command line or a committed file.</summary>
    public required string Token { get; init; }

    /// <summary>The one guild the bot serves. Its commands are registered there, which takes effect at once.</summary>
    public required ulong Guild { get; init; }

    /// <summary>The deck every seat plays, by name, for the messages.</summary>
    public required string Deck { get; init; }

    /// <summary>
    /// The web client every ticket reply opens (<see cref="ClientSite"/>): the bot's own by default, or the same static
    /// client served from anywhere else.
    /// </summary>
    public required Uri Client { get; init; }

    /// <summary>Where card art is, for the client's <c>?art=</c>; null leaves the client to its default.</summary>
    public Uri? Art { get; init; }

    /// <summary>The cube drafts draft, by name, for the messages; null when the bot hosts no drafts.</summary>
    public string? Cube { get; init; }

    /// <summary>The draft queue page (<see cref="QueueSite"/>), which every queue's message links to; null when the bot serves none.</summary>
    public Uri? QueuePage { get; init; }

    /// <summary>
    /// Where each player's Discord picture is kept, taken afresh whenever they join a draft; null to keep none.
    /// </summary>
    public PortraitStore? Portraits { get; init; }
}

/// <summary>
/// The Discord side: <c>/cantrip host</c>, and the Join and Cancel buttons on each game's message. Everything it
/// decides, it asks the <see cref="Matchmaker"/>; everything it says comes from <see cref="GameMessages"/>.
/// <para>
/// It connects over the gateway, so the machine running it needs no public HTTP endpoint for Discord, and asks for the
/// Guilds intent only — no privileged intents, no message content. Tickets go to their owner alone, in ephemeral
/// replies. Handlers run off the gateway thread, because hosting can take seconds and Discord.Net warns, then falls
/// behind on heartbeats, when a handler blocks it.
/// </para>
/// </summary>
public sealed class DiscordBot : IAsyncDisposable
{
    private const string JoinPrefix = "cantrip:join:";
    private const string CancelPrefix = "cantrip:cancel:";
    private const string DraftJoinPrefix = "cantrip:draft-join:";
    private const string DraftCancelPrefix = "cantrip:draft-cancel:";
    private const string QueueJoinPrefix = "cantrip:queue-join:";

    private readonly DiscordBotOptions _options;
    private readonly Matchmaker _matchmaker;
    private readonly DraftHost? _drafts;
    private readonly PodHosts? _pods;
    private readonly DraftQueues? _queues;
    private readonly ConcurrentDictionary<string, IUserMessage> _draftMessages = new(StringComparer.Ordinal);
    private readonly HostPolicy _policy;
    private readonly DiscordSocketClient _client;
    private readonly ConcurrentDictionary<string, IUserMessage> _messages = new(StringComparer.Ordinal);
    private readonly SemaphoreSlim _rendering = new(1, 1);
    private int _registered;

    public DiscordBot(DiscordBotOptions options, Matchmaker matchmaker, HostPolicy policy, DraftHost? drafts = null, DraftQueues? queues = null, PodHosts? pods = null)
    {
        _options = options;
        _matchmaker = matchmaker;
        _drafts = drafts;
        _pods = pods;
        _queues = queues;
        _policy = policy;
        _client = new DiscordSocketClient(new DiscordSocketConfig { GatewayIntents = GatewayIntents.Guilds });
        _client.Log += message =>
        {
            Console.WriteLine($"[discord] {message}");
            return Task.CompletedTask;
        };
        _client.Ready += RegisterAsync;
        _client.SlashCommandExecuted += command => Detach(() => CommandAsync(command), command);
        _client.ButtonExecuted += button => Detach(() => ButtonAsync(button), button);
        _matchmaker.Changed += RenderAsync;
        if (_drafts is not null) _drafts.Changed += RenderDraftAsync;
        if (_queues is not null)
        {
            _queues.Changed += RenderQueueAsync;

            // A draft from a queue starting or ending changes the queue's count of drafts under way.
            if (_drafts is not null) _drafts.Changed += draft => draft.Queue == 0 ? Task.CompletedTask : RenderQueueAsync(draft.Queue);
            if (_pods is not null) _pods.Changed += queue => queue == 0 ? Task.CompletedTask : RenderQueueAsync(queue);
        }
    }

    public async Task StartAsync()
    {
        await _client.LoginAsync(TokenType.Bot, _options.Token).ConfigureAwait(false);
        await _client.StartAsync().ConfigureAwait(false);
    }

    /// <summary>Registers the commands in the guild, once: Ready is raised again on every reconnect.</summary>
    private async Task RegisterAsync()
    {
        if (Interlocked.Exchange(ref _registered, 1) == 1) return;
        if (_client.GetGuild(_options.Guild) is not { } guild)
        {
            Console.Error.WriteLine($"The bot is not in guild {_options.Guild}. Invite it there, then restart.");
            _registered = 0;
            return;
        }

        SlashCommandBuilder builder = new SlashCommandBuilder()
            .WithName("cantrip")
            .WithDescription("Cantrip games")
            .WithContextTypes(InteractionContextType.Guild)
            .AddOption(new SlashCommandOptionBuilder()
                .WithName("host")
                .WithDescription($"Host a two-player game ({_options.Deck} for both seats)")
                .WithType(ApplicationCommandOptionType.SubCommand));
        if (_drafts is not null)
        {
            builder.AddOption(new SlashCommandOptionBuilder()
                .WithName("draft")
                .WithDescription($"Open a two-player Winston draft of {_options.Cube}, then play with the decks you build")
                .WithType(ApplicationCommandOptionType.SubCommand));
            builder.AddOption(new SlashCommandOptionBuilder()
                .WithName("mydraft")
                .WithDescription("A fresh link to the draft you are in, from any computer")
                .WithType(ApplicationCommandOptionType.SubCommand));
        }

        if (_queues is not null)
        {
            builder.AddOption(new SlashCommandOptionBuilder()
                .WithName("queue")
                .WithDescription("Make this channel a draft queue, or change what it drafts (server managers)")
                .WithType(ApplicationCommandOptionType.SubCommand)
                .AddOption(new SlashCommandOptionBuilder().WithName("cube").WithDescription("The cube, by name (LSVCube, PowerLSV)").WithType(ApplicationCommandOptionType.String).WithRequired(true))
                .AddOption(new SlashCommandOptionBuilder().WithName("format").WithDescription("The kind of draft").WithType(ApplicationCommandOptionType.String).WithRequired(false).AddChoice("Winston", "winston").AddChoice("Booster (pod of 8)", "booster").AddChoice("Grid", "grid"))
                .AddOption(new SlashCommandOptionBuilder().WithName("cards").WithDescription("How many cards a draft takes from the cube (90; 0 for all)").WithType(ApplicationCommandOptionType.Integer).WithRequired(false))
                .AddOption(new SlashCommandOptionBuilder().WithName("piles").WithDescription("How many piles (3)").WithType(ApplicationCommandOptionType.Integer).WithRequired(false))
                .AddOption(new SlashCommandOptionBuilder().WithName("packs").WithDescription("Booster: packs each player opens (3); grid: grids laid out (18)").WithType(ApplicationCommandOptionType.Integer).WithRequired(false))
                .AddOption(new SlashCommandOptionBuilder().WithName("pack-size").WithDescription("Booster: cards in a pack (15)").WithType(ApplicationCommandOptionType.Integer).WithRequired(false))
                .AddOption(new SlashCommandOptionBuilder().WithName("rounds").WithDescription("Booster: Swiss rounds played (3)").WithType(ApplicationCommandOptionType.Integer).WithRequired(false))
                .AddOption(new SlashCommandOptionBuilder().WithName("min-deck").WithDescription("The fewest cards a deck may have (40)").WithType(ApplicationCommandOptionType.Integer).WithRequired(false))
                .AddOption(new SlashCommandOptionBuilder().WithName("bots").WithDescription("Seats played by bots, so fewer players fill a table (0)").WithType(ApplicationCommandOptionType.Integer).WithRequired(false)));
            builder.AddOption(new SlashCommandOptionBuilder()
                .WithName("unqueue")
                .WithDescription("Close this channel's draft queue (server managers)")
                .WithType(ApplicationCommandOptionType.SubCommand));
        }
        SlashCommandProperties cantrip = builder.Build();
        await guild.BulkOverwriteApplicationCommandAsync([cantrip]).ConfigureAwait(false);
        Console.WriteLine($"Registered /cantrip in {guild.Name}.");

        // Every queue's message brought up to date, or posted again if it is gone.
        if (_queues is not null)
        {
            foreach (QueueSettings queue in _queues.Queues)
            {
                await RenderQueueAsync(queue.Channel).ConfigureAwait(false);
            }
        }
    }

    private Task Detach(Func<Task> handle, SocketInteraction interaction)
    {
        _ = Task.Run(async () =>
        {
            try
            {
                await handle().ConfigureAwait(false);
            }
            catch (Exception error)
            {
                Console.Error.WriteLine($"An interaction failed: {error}");
                try
                {
                    const string sorry = "Something went wrong on the bot's side. The host's log has the details.";
                    if (interaction.HasResponded) await interaction.FollowupAsync(sorry, ephemeral: true).ConfigureAwait(false);
                    else await interaction.RespondAsync(sorry, ephemeral: true).ConfigureAwait(false);
                }
                catch (Exception)
                {
                    // The interaction may have expired; the error is in the log either way.
                }
            }
        });
        return Task.CompletedTask;
    }

    private async Task CommandAsync(SocketSlashCommand command)
    {
        string? sub = command.Data.Options.FirstOrDefault()?.Name;
        if (command.Data.Name != "cantrip" || sub is not ("host" or "draft" or "mydraft" or "queue" or "unqueue")) return;
        if (sub is "queue" or "unqueue" && _queues is not null)
        {
            await QueueCommandAsync(command, sub).ConfigureAwait(false);
            return;
        }

        if (sub == "mydraft" && _drafts is not null)
        {
            // Anyone may ask for their own seat back; nothing is hosted, so no host role is needed.
            await command.DeferAsync(ephemeral: true).ConfigureAwait(false);
            if (_pods is not null && await _pods.ReturnAsync(Player(command.User)).ConfigureAwait(false) is { Refusal: null } pod)
            {
                MessageComponent link = new ComponentBuilder()
                    .WithButton("Open in Cantrip", style: ButtonStyle.Link, url: ClientSite.Link(_options.Client, pod.Address!, pod.Ticket!, _options.Art).AbsoluteUri)
                    .Build();
                await command.FollowupAsync(GameMessages.PodTicket(pod), ephemeral: true, components: link).ConfigureAwait(false);
                return;
            }

            DraftSeating back = await _drafts.ReturnAsync(Player(command.User)).ConfigureAwait(false);
            if (back.Refusal is not null) await command.FollowupAsync(back.Refusal, ephemeral: true).ConfigureAwait(false);
            else await command.FollowupAsync(GameMessages.DraftTicket(back), ephemeral: true, components: OpenDraft(_options, back)).ConfigureAwait(false);
            return;
        }

        if (command.GuildId != _options.Guild || command.User is not SocketGuildUser member)
        {
            await command.RespondAsync("Host games from a channel in the Cantrip server.", ephemeral: true).ConfigureAwait(false);
            return;
        }
        if (!_policy.MayHost(member.Roles.Select(role => role.Id), member.GuildPermissions.ManageGuild))
        {
            await command.RespondAsync(_policy.Refusal, ephemeral: true, allowedMentions: AllowedMentions.None).ConfigureAwait(false);
            return;
        }

        // Opening a table can take longer than the three seconds Discord allows for a first response.
        await command.DeferAsync(ephemeral: true).ConfigureAwait(false);
        if (sub == "draft")
        {
            await OpenDraftAsync(command, member).ConfigureAwait(false);
            return;
        }

        Seating seating = await _matchmaker.HostAsync(Player(member)).ConfigureAwait(false);
        if (seating.Refusal is not null)
        {
            await command.FollowupAsync(seating.Refusal, ephemeral: true).ConfigureAwait(false);
            return;
        }

        HostedGame game = seating.Game!;
        try
        {
            IMessageChannel channel = command.Channel
                ?? await _client.GetChannelAsync(command.ChannelId!.Value).ConfigureAwait(false) as IMessageChannel
                ?? throw new InvalidOperationException("The command's channel is not a message channel.");
            IUserMessage message = await channel.SendMessageAsync(
                GameMessages.Table(game, _options.Deck),
                allowedMentions: AllowedMentions.None,
                components: Buttons(game)).ConfigureAwait(false);
            _messages[game.Id] = message;
        }
        catch (Exception error) when (error is HttpException or InvalidOperationException)
        {
            // A game nobody can see cannot be joined. Take it down rather than leave it holding a port.
            await _matchmaker.CancelAsync(game.Id, game.Host, moderator: true).ConfigureAwait(false);
            await command.FollowupAsync(
                $"The game could not be posted in this channel, so it was cancelled. Does the bot have Send Messages here? ({error.Message})",
                ephemeral: true).ConfigureAwait(false);
            return;
        }

        await command.FollowupAsync(GameMessages.Ticket(seating), ephemeral: true, components: Open(_options, seating)).ConfigureAwait(false);
    }

    /// <summary>Opens a draft and posts its message, which the second player joins from.</summary>
    private async Task OpenDraftAsync(SocketSlashCommand command, SocketGuildUser member)
    {
        if (_drafts is null) return;
        KeepPortrait(member);
        DraftSeating seating = await _drafts.OpenAsync(Player(member)).ConfigureAwait(false);
        if (seating.Refusal is not null)
        {
            await command.FollowupAsync(seating.Refusal, ephemeral: true).ConfigureAwait(false);
            return;
        }

        HostedDraft draft = seating.Draft!;
        try
        {
            IMessageChannel channel = command.Channel
                ?? await _client.GetChannelAsync(command.ChannelId!.Value).ConfigureAwait(false) as IMessageChannel
                ?? throw new InvalidOperationException("The command's channel is not a message channel.");
            IUserMessage message = await channel.SendMessageAsync(
                GameMessages.Draft(draft, _options.Cube ?? string.Empty),
                allowedMentions: AllowedMentions.None,
                components: DraftButtons(draft)).ConfigureAwait(false);
            _draftMessages[draft.Id] = message;
            await _drafts.NoteMessageAsync(draft, $"{channel.Id}:{message.Id}").ConfigureAwait(false);
        }
        catch (Exception error) when (error is HttpException or InvalidOperationException)
        {
            await _drafts.CancelAsync(draft.Id, draft.Host, moderator: true).ConfigureAwait(false);
            await command.FollowupAsync(
                $"The draft could not be posted in this channel, so it was cancelled. Does the bot have Send Messages here? ({error.Message})",
                ephemeral: true).ConfigureAwait(false);
            return;
        }

        await command.FollowupAsync(
            "Your draft is open. When someone presses **Join draft**, press **My ticket** on it to open your seat.",
            ephemeral: true).ConfigureAwait(false);
    }

    /// <summary>Sets up, changes or closes the channel's queue. Server managers only: the queue is the server's.</summary>
    private async Task QueueCommandAsync(SocketSlashCommand command, string sub)
    {
        if (command.GuildId != _options.Guild || command.User is not SocketGuildUser { GuildPermissions.ManageGuild: true })
        {
            await command.RespondAsync("Only a server manager can set up a draft queue.", ephemeral: true).ConfigureAwait(false);
            return;
        }

        await command.DeferAsync(ephemeral: true).ConfigureAwait(false);
        ulong channel = command.ChannelId!.Value;
        if (sub == "unqueue")
        {
            QueueSettings? closed = await _queues!.RemoveAsync(channel).ConfigureAwait(false);
            if (closed is null)
            {
                await command.FollowupAsync("This channel has no draft queue.", ephemeral: true).ConfigureAwait(false);
                return;
            }

            if (await FindAsync($"{channel}:{closed.Message}").ConfigureAwait(false) is { } message)
            {
                await message.ModifyAsync(properties =>
                {
                    properties.Content = "This draft queue is closed.";
                    properties.Components = new ComponentBuilder().Build();
                }).ConfigureAwait(false);
            }

            await command.FollowupAsync("The queue is closed. Tables still forming were closed; drafts under way carry on.", ephemeral: true).ConfigureAwait(false);
            return;
        }

        SocketSlashCommandDataOption options = command.Data.Options.First();
        object? Option(string name) => options.Options.FirstOrDefault(option => option.Name == name)?.Value;
        string format = Option("format") as string ?? DraftFormats.Winston;
        var settings = new QueueSettings
        {
            Channel = channel,
            Cube = (string)Option("cube")!,
            Format = format,
            Cards = Option("cards") is long cards ? (int)cards : 90,
            Piles = Option("piles") is long piles ? (int)piles : 3,
            Packs = Option("packs") is long packs ? (int)packs : 0,
            PackSize = Option("pack-size") is long packSize ? (int)packSize : 15,
            Rounds = Option("rounds") is long rounds ? (int)rounds : 3,
            MinimumDeck = Option("min-deck") is long minimum ? (int)minimum : 40,
            Bots = Option("bots") is long bots ? (int)bots : 0,
        };
        if (await _queues!.ConfigureAsync(settings).ConfigureAwait(false) is { } refusal)
        {
            await command.FollowupAsync(refusal, ephemeral: true).ConfigureAwait(false);
            return;
        }

        // Said only once its message, with Join, is up: a queue nobody can see a way into is not set up yet.
        if (await PostQueueAsync(channel).ConfigureAwait(false) is { } failed)
        {
            await command.FollowupAsync($"The queue is saved, but its message is not up yet. {failed}", ephemeral: true).ConfigureAwait(false);
            return;
        }

        string kind = settings.Format switch
        {
            DraftFormats.Booster => "booster draft (pod of 8)",
            DraftFormats.Grid => "grid draft",
            _ => "Winston draft",
        };
        string withBots = settings.Bots switch
        {
            0 => string.Empty,
            1 => ", with a bot in one seat",
            _ => $", with bots in {settings.Bots} seats",
        };
        await command.FollowupAsync($"This channel is now a {kind} queue for {_queues.CubeName(channel)}{withBots}.", ephemeral: true).ConfigureAwait(false);
    }

    /// <summary>
    /// Brings a queue's message up to date: edited where it is, or posted if it never was or has been deleted, and kept
    /// with the queue so a restart finds it.
    /// </summary>
    private Task RenderQueueAsync(ulong channel) => PostQueueAsync(channel);

    /// <summary>The same, saying why the message could not be posted or edited; null once it is up.</summary>
    private async Task<string?> PostQueueAsync(ulong channel)
    {
        if (_queues?.Status(channel) is not { } status) return "This channel has no draft queue.";
        await _rendering.WaitAsync().ConfigureAwait(false);
        try
        {
            string content = GameMessages.Queue(status, _queues.CubeName(channel));
            var buttons = new ComponentBuilder().WithButton("Join", QueueJoinPrefix + channel, ButtonStyle.Primary);
            if (_options.QueuePage is { } page) buttons.WithButton("Every table", style: ButtonStyle.Link, url: page.AbsoluteUri);
            MessageComponent join = buttons.Build();
            if (await FindAsync($"{channel}:{status.Settings.Message}").ConfigureAwait(false) is { } message)
            {
                await message.ModifyAsync(properties =>
                {
                    properties.Content = content;
                    properties.Components = join;
                    properties.AllowedMentions = AllowedMentions.None;
                }).ConfigureAwait(false);
                return null;
            }

            if (await _client.GetChannelAsync(channel).ConfigureAwait(false) is not IMessageChannel place)
                return "The bot cannot see this channel. Give it View Channel here.";

            IUserMessage posted = await place.SendMessageAsync(content, allowedMentions: AllowedMentions.None, components: join).ConfigureAwait(false);
            await _queues.NoteMessageAsync(channel, posted.Id).ConfigureAwait(false);
            return null;
        }
        catch (Exception error)
        {
            Console.Error.WriteLine($"Could not update the queue message in channel {channel}: {error.Message}");
            return error is HttpException { HttpCode: System.Net.HttpStatusCode.Forbidden }
                ? "The bot is not allowed to post in this channel. Give it View Channel and Send Messages here (check the category's permissions too), then run /cantrip queue again."
                : $"The queue's message could not be posted: {error.Message}";
        }
        finally
        {
            _rendering.Release();
        }
    }

    private async Task ButtonAsync(SocketMessageComponent button)
    {
        string id = button.Data.CustomId;
        if (_queues is not null && id.StartsWith(QueueJoinPrefix, StringComparison.Ordinal)
            && ulong.TryParse(id[QueueJoinPrefix.Length..], out ulong queueChannel))
        {
            await button.DeferAsync(ephemeral: true).ConfigureAwait(false);
            KeepPortrait(button.User);
            QueueSeating seating = await _queues.JoinAsync(queueChannel, Player(button.User)).ConfigureAwait(false);
            if (seating.Refusal is not null)
            {
                await button.FollowupAsync(seating.Refusal, ephemeral: true).ConfigureAwait(false);
            }
            else
            {
                MessageComponent open = new ComponentBuilder()
                    .WithButton("Open in Cantrip", style: ButtonStyle.Link,
                        url: ClientSite.Link(_options.Client, seating.Address!, seating.Ticket!, _options.Art).AbsoluteUri)
                    .Build();
                await button.FollowupAsync(GameMessages.QueueTicket(seating), ephemeral: true, components: open).ConfigureAwait(false);
            }
        }
        else if (_drafts is not null && id.StartsWith(DraftJoinPrefix, StringComparison.Ordinal))
        {
            await button.DeferAsync(ephemeral: true).ConfigureAwait(false);
            KeepPortrait(button.User);
            DraftSeating seating = await _drafts.JoinAsync(id[DraftJoinPrefix.Length..], Player(button.User)).ConfigureAwait(false);
            if (seating.Refusal is not null)
                await button.FollowupAsync(seating.Refusal, ephemeral: true).ConfigureAwait(false);
            else
                await button.FollowupAsync(GameMessages.DraftTicket(seating), ephemeral: true, components: OpenDraft(_options, seating)).ConfigureAwait(false);
        }
        else if (_drafts is not null && id.StartsWith(DraftCancelPrefix, StringComparison.Ordinal))
        {
            await button.DeferAsync(ephemeral: true).ConfigureAwait(false);
            bool manager = button.User is SocketGuildUser { GuildPermissions.ManageGuild: true };
            string? refusal = await _drafts.CancelAsync(id[DraftCancelPrefix.Length..], Player(button.User), manager).ConfigureAwait(false);
            await button.FollowupAsync(refusal ?? "Cancelled.", ephemeral: true).ConfigureAwait(false);
        }
        else if (id.StartsWith(JoinPrefix, StringComparison.Ordinal))
        {
            await button.DeferAsync(ephemeral: true).ConfigureAwait(false);
            Seating seating = await _matchmaker.JoinAsync(id[JoinPrefix.Length..], Player(button.User)).ConfigureAwait(false);
            if (seating.Refusal is not null)
                await button.FollowupAsync(seating.Refusal, ephemeral: true).ConfigureAwait(false);
            else
                await button.FollowupAsync(GameMessages.Ticket(seating), ephemeral: true, components: Open(_options, seating)).ConfigureAwait(false);
        }
        else if (id.StartsWith(CancelPrefix, StringComparison.Ordinal))
        {
            await button.DeferAsync(ephemeral: true).ConfigureAwait(false);
            bool manager = button.User is SocketGuildUser { GuildPermissions.ManageGuild: true };
            string? refusal = await _matchmaker.CancelAsync(id[CancelPrefix.Length..], Player(button.User), manager).ConfigureAwait(false);
            await button.FollowupAsync(refusal ?? "Cancelled.", ephemeral: true).ConfigureAwait(false);
        }
    }

    /// <summary>Brings a game's public message up to date, and deletes it once the game has ended.</summary>
    private async Task RenderAsync(HostedGame game)
    {
        // One edit at a time, each drawn from the game as it is then rather than as it was when the change was raised,
        // so a slow "in progress" edit can never land after the deletion and post nothing back.
        await _rendering.WaitAsync().ConfigureAwait(false);
        try
        {
            if (!_messages.TryGetValue(game.Id, out IUserMessage? message)) return;
            if (game.Stage == GameStage.Ended)
            {
                _messages.TryRemove(game.Id, out _);
                await DeleteAsync(message, $"game {game.Id}").ConfigureAwait(false);
            }
            else
            {
                await EditAsync(game, message).ConfigureAwait(false);
            }
        }
        finally
        {
            _rendering.Release();
        }
    }

    private async Task EditAsync(HostedGame game, IUserMessage message)
    {
        try
        {
            await message.ModifyAsync(properties =>
            {
                properties.Content = GameMessages.Table(game, _options.Deck);
                properties.Components = Buttons(game);
                properties.AllowedMentions = AllowedMentions.None;
            }).ConfigureAwait(false);
        }
        catch (Exception error)
        {
            // Someone deleted the message, the bot lost the channel, or Discord is down. The game goes on regardless,
            // and a player waiting on a Join reply still gets their ticket: the matchmaker awaits this before replying.
            Console.Error.WriteLine($"Could not update the message for game {game.Id}: {error.Message}");
        }
    }

    /// <summary>
    /// Brings a draft's public message up to date, and deletes it once the draft is over. After a restart the bot has no
    /// hold on the message, so it finds it again from where the event kept it.
    /// </summary>
    private async Task RenderDraftAsync(HostedDraft draft)
    {
        await _rendering.WaitAsync().ConfigureAwait(false);
        try
        {
            IUserMessage? message = _draftMessages.GetValueOrDefault(draft.Id) ?? await FindAsync(draft.Message).ConfigureAwait(false);
            if (message is null) return;
            if (!draft.IsLive)
            {
                _draftMessages.TryRemove(draft.Id, out _);
                await DeleteAsync(message, $"draft {draft.Id}").ConfigureAwait(false);
                return;
            }

            _draftMessages[draft.Id] = message;
            await message.ModifyAsync(properties =>
            {
                properties.Content = GameMessages.Draft(draft, _options.Cube ?? string.Empty);
                properties.Components = DraftButtons(draft);
                properties.AllowedMentions = AllowedMentions.None;
            }).ConfigureAwait(false);
        }
        catch (Exception error)
        {
            Console.Error.WriteLine($"Could not update the message for draft {draft.Id}: {error.Message}");
        }
        finally
        {
            _rendering.Release();
        }
    }

    /// <summary>
    /// Takes down the public message of a game or draft that is over: its players have their result at the table, and
    /// the channel keeps only what can still be joined.
    /// </summary>
    private static async Task DeleteAsync(IUserMessage message, string what)
    {
        try
        {
            await message.DeleteAsync().ConfigureAwait(false);
        }
        catch (HttpException error) when (error.HttpCode == System.Net.HttpStatusCode.NotFound)
        {
            // Someone deleted it first.
        }
        catch (Exception error)
        {
            Console.Error.WriteLine($"Could not delete the message for {what}: {error.Message}");
        }
    }

    /// <summary>A message by where it was kept, "channel:message"; null when it is gone or was never kept.</summary>
    private async Task<IUserMessage?> FindAsync(string? kept)
    {
        // A message never posted is kept as id 0, which Discord answers with an error rather than nothing.
        if (kept?.Split(':') is not [{ } channelId, { } messageId]
            || !ulong.TryParse(channelId, out ulong channel) || !ulong.TryParse(messageId, out ulong id) || id == 0)
            return null;

        try
        {
            return await _client.GetChannelAsync(channel).ConfigureAwait(false) is IMessageChannel found
                ? await found.GetMessageAsync(id).ConfigureAwait(false) as IUserMessage
                : null;
        }
        catch (HttpException error) when (error.HttpCode is System.Net.HttpStatusCode.NotFound or System.Net.HttpStatusCode.BadRequest)
        {
            // Deleted since, or never there: to be posted again.
            return null;
        }
    }

    /// <summary>Join while the second seat is free, then My ticket for either player, at any point; Cancel until it ends.</summary>
    private static MessageComponent DraftButtons(HostedDraft draft) => !draft.IsLive
        ? new ComponentBuilder().Build()
        : new ComponentBuilder()
            .WithButton(draft.Guest is null ? "Join draft" : "My ticket", DraftJoinPrefix + draft.Id, draft.Guest is null ? ButtonStyle.Primary : ButtonStyle.Secondary)
            .WithButton("Cancel", DraftCancelPrefix + draft.Id, ButtonStyle.Danger)
            .Build();

    private static MessageComponent OpenDraft(DiscordBotOptions options, DraftSeating seating) => new ComponentBuilder()
        .WithButton("Open in Cantrip", style: ButtonStyle.Link,
            url: ClientSite.Link(options.Client, seating.Draft!.Address!, seating.Ticket!, options.Art).AbsoluteUri)
        .Build();

    /// <summary>
    /// Join while there is a seat; once both are taken the same button hands a seated player their ticket again,
    /// which is how someone whose client closed gets back in. Cancel until it ends.
    /// </summary>
    private static MessageComponent Buttons(HostedGame game) => game.Stage switch
    {
        GameStage.Ended => new ComponentBuilder().Build(),
        GameStage.Open => new ComponentBuilder()
            .WithButton("Join", JoinPrefix + game.Id, ButtonStyle.Primary)
            .WithButton("Cancel", CancelPrefix + game.Id, ButtonStyle.Danger)
            .Build(),
        _ => new ComponentBuilder()
            .WithButton("My ticket", JoinPrefix + game.Id, ButtonStyle.Secondary)
            .WithButton("Cancel", CancelPrefix + game.Id, ButtonStyle.Danger)
            .Build(),
    };

    /// <summary>The link button on a ticket reply, which opens the web client in the player's seat at the table.</summary>
    private static MessageComponent Open(DiscordBotOptions options, Seating seating) => new ComponentBuilder()
        .WithButton("Open in Cantrip", style: ButtonStyle.Link,
            url: ClientSite.Link(options.Client, seating.Game!.Address, seating.Ticket!, options.Art).AbsoluteUri)
        .Build();

    /// <summary>
    /// The player an account signed in on the queue page sits down as: by their name on this server, with their picture
    /// kept, as a Join here does - or, for someone not on the server, by their account's own name and avatar.
    /// </summary>
    public async Task<DiscordPlayer> WebPlayerAsync(WebAccount account)
    {
        ArgumentNullException.ThrowIfNull(account);
        IGuildUser? member = null;
        try
        {
            member = await _client.Rest.GetGuildUserAsync(_options.Guild, account.Id).ConfigureAwait(false);
        }
        catch (HttpException error)
        {
            Console.Error.WriteLine($"[discord] Could not look {account.Id} up on the server: {error.Message}");
        }

        if (member is not null)
        {
            KeepPortrait(member);
            return Player(member);
        }

        KeepPortrait(account.Id, account.Avatar is { } avatar
            ? CDN.GetUserAvatarUrl(account.Id, avatar, PortraitSize, ImageFormat.Png)
            : CDN.GetDefaultUserAvatarUrl(account.Id));
        return new DiscordPlayer(account.Id, account.Name);
    }

    /// <summary>A channel's name as the queue page shows it, <c>#booster</c>; null for one the bot cannot see.</summary>
    public string? ChannelName(ulong channel) => _client.GetChannel(channel) is IGuildChannel { Name: { } name } ? "#" + name : null;

    private static DiscordPlayer Player(IUser user) =>
        new(user.Id, user is IGuildUser { DisplayName: { } shown } ? shown : user.GlobalName ?? user.Username);

    /// <summary>How large a picture is taken: twice the rail's portrait, for a sharp one on a high-density screen.</summary>
    private const ushort PortraitSize = 128;

    private static readonly HttpClient Pictures = new() { Timeout = TimeSpan.FromSeconds(15) };

    /// <summary>
    /// Takes the player's picture - their avatar on this server, else their own, else Discord's stand-in - and keeps it as
    /// their portrait in place of the last. Started as they join, alongside it rather than ahead of it: a picture that
    /// cannot be had now leaves the one kept, and never holds up a seat.
    /// </summary>
    private void KeepPortrait(IUser user) =>
        KeepPortrait(user.Id, (user as IGuildUser)?.GetGuildAvatarUrl(ImageFormat.Png, PortraitSize) ?? user.GetDisplayAvatarUrl(ImageFormat.Png, PortraitSize));

    private void KeepPortrait(ulong user, string url)
    {
        if (_options.Portraits is not { } portraits) return;
        string player = DraftHost.PlayerId(new DiscordPlayer(user, string.Empty));
        _ = Task.Run(async () =>
        {
            try
            {
                await portraits.SaveAsync(player, await Pictures.GetByteArrayAsync(url).ConfigureAwait(false)).ConfigureAwait(false);
            }
            catch (Exception error) when (error is HttpRequestException or TaskCanceledException or IOException)
            {
                Console.Error.WriteLine($"[discord] No picture of {player} for now: {error.Message}");
            }
        });
    }

    public async ValueTask DisposeAsync()
    {
        _matchmaker.Changed -= RenderAsync;
        if (_drafts is not null) _drafts.Changed -= RenderDraftAsync;
        if (_queues is not null) _queues.Changed -= RenderQueueAsync;
        await _client.StopAsync().ConfigureAwait(false);
        await _client.LogoutAsync().ConfigureAwait(false);
        await _client.DisposeAsync().ConfigureAwait(false);
    }
}

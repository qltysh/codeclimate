require "cc/cli/command"

module CC
  module CLI
    class Analyze < Command
      ARGUMENT_LIST = "[-f format] [-e engine[:channel]] [path]".freeze
      SHORT_HELP = "Run analysis with the given arguments".freeze
      HELP = "#{SHORT_HELP}\n" \
        "\n" \
        "    -f <format>, --format <format>   Format of output. Possible values: #{CC::Analyzer::Formatters::FORMATTERS.keys.join ", "}\n" \
        "    -e <engine[:channel]>            Engine to run. Can be specified multiple times.\n" \
        "    --dev                            Run in development mode. Engines installed locally that are not in the manifest will be run.\n" \
        "    path                             Path to check. Can be specified multiple times.".freeze

      EngineFailure = Class.new(StandardError)

      include CC::Analyzer

      def run
        Dir.chdir(MountedPath.code.container_path) do
          # Load config here so it sees ./.codeclimate.yml
          @config = Config.load

          # process args after, so it modifies loaded configuration
          process_args

          bridge = Bridge.new(
            config: config,
            formatter: formatter,
            listener: CompositeContainerListener.new(
              LoggingContainerListener.new(Analyzer.logger),
              RaisingContainerListener.new(EngineFailure),
            ),
            registry: EngineRegistry.new
          )

          bridge.run
        end
      end

      private

      attr_reader :config, :listener, :registry

      def process_args
        while (arg = @args.shift)
          case arg
          when "-f"
            @formatter = Formatters.resolve(@args.shift).new(filesystem)
          when "-e", "--engine"
            # First time passed, clear any configured engines
            unless @cleared
              config.engines.clear
              @cleared = true
            end
            name, channel = @args.shift.split(":", 2)
            config.engines << Config::Engine.new(
              name,
              channel: channel,
              enabled: true,
            )
          when "--dev"
            config.development = true
          else
            config.analysis_paths << arg
          end
        end
      rescue Formatters::Formatter::InvalidFormatterError => ex
        fatal(e.message)
      end

      def formatter
        @formatter ||= Formatters::PlainTextFormatter.new(filesystem)
      end
    end
  end
end

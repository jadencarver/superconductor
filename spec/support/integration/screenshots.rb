require 'ansi'
require 'mini_magick'

module IntegrationSpec
  module Screenshots
    extend RSpec::Matchers::DSL

    SIZE = 18
    WRAP = true
    PATH = "spec/integration/screenshots"

    def tracked_screenshots
      @@tracked_screenshots ||= begin
                                  Hash[`git ls-files -s spec/features/screenshots`.split("\n").map do |line|
                                    key, *values = line.split(/\s/).reverse; [key, values]
                                  end]
                                end
    end

    def take_screenshot(options = {})
      options = { options => true } if options.is_a?(Symbol)
      page.save_screenshot(nil, options).tap do |path|
        term_width, max_width = ANSI::Terminal.terminal_width, 125
        if term_width > max_width then width = max_width
        else width = term_width
        end

        data = Base64.encode64(File.read(path))
        puts "\n#{ANSI.right(term_width/2-width/2)}\033]1337;File=;inline=1;width=#{width}:#{data}\a\n\n"
      end
    end
    alias_method :tk, :take_screenshot

    matcher :look_like do |name|
      passed = true
      printf ANSI.clear_line + "\n"
      match do |page_or_element|
        if page_or_element.is_a?(Capybara::Node::Element)
          element = page_or_element
          page = page_or_element.session
        else
          page = page_or_element
        end
        [
          #["#{name}_iphone5.png", "iPhone 5", 320, 568 ],
          #["#{name}_iphone6plus.png", "iPhone 6+", 414, 736 ],
          #["#{name}_ipad.png", "iPad Portrait", 768, 1024],
          ["#{name}.png", "Desktop", 2880/2, 1800/2 ],
        ].inject(0) do |accum, (filename, caption, width, height)|

          path = "#{PATH}/#{filename}"
          term_width = (width/height.to_f * (SIZE * 2.1)).ceil
          break if !WRAP && accum + term_width > ANSI::Terminal.terminal_width
          #page.driver.resize(width, height)

          page.save_screenshot path

          if element
            crop = page.evaluate_script(<<-JS)
              document.evaluate(
                '#{element.path}',
                document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null
              ).singleNodeValue.getBoundingClientRect()
            JS
            screenshot = MiniMagick::Image.open(path)
            screenshot.crop "%{width}x%{height}+%{left}+%{top}" % crop.inject({}) { |h,(k,v)| h[k.to_sym]=v; h }
            screenshot.write path
          else
            page.save_screenshot path
          end

          git_status = tracked_screenshots[path]

          if git_status && git_status[1] != `git hash-object #{path}`.chomp
            # BAD
            passed = false
            system("mkdir -p tmp/capybara")
            system("git cat-file blob #{git_status[1]} | compare - #{path} tmp/capybara/diff_#{filename}")
            system("git cat-file blob #{git_status[1]} | convert -delay 150 -resize x300 -loop 0 - -background Orange label:'After' -gravity Center -append #{path} tmp/capybara/diff_#{filename} tmp/capybara/diff_#{filename}.gif")

            data = Base64.encode64(File.read("tmp/capybara/diff_#{filename}.gif"))
          else
            # GOOD
            data = Base64.encode64(File.read(path))
          end

          print ANSI.up(SIZE+1) + ANSI.right(accum) if accum > 0
          puts "\033]1337;File=;inline=1;height=#{SIZE}:#{data}\a\n"
          print ANSI.right(accum) if accum > 0
          puts ANSI.bold { caption.center(term_width) }

          accum + term_width
        end
        puts
        return true if passed
      end
    end

  end
end

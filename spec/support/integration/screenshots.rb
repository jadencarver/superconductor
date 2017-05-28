module IntegrationSpec::Screenshots
  extend RSpec::Matchers::DSL
  
  def tracked_screenshots
    @@tracked_screenshots ||= begin
                            Hash[`git ls-files -s spec/features/screenshots`.split("\n").map do |line|
                              key, *values = line.split(/\s/).reverse; [key, values]
                            end]
                          end
  end

  matcher :take_screenshot_of do |name|
    passed = true
    printf ANSI.clear_line + "\n"
    match do |page|
      [
        ["spec/features/screenshots/#{name.parameterize}.png", "Desktop", 2880/2, 1800/2 ],
        ["spec/features/screenshots/#{name.parameterize}_ipad.png", "iPad Portrait", 768, 1024],
        ["spec/features/screenshots/#{name.parameterize}_iphone6plus.png", "iPhone 6+", 414, 736 ],
        ["spec/features/screenshots/#{name.parameterize}_iphone5.png", "iPhone 5", 320, 568 ],
      ].inject(0) do |accum, (path, name, width, height)|
        term_width = (width/height.to_f * (18 * 2.33)).ceil
        # break if config.accum + term_width > ANSI::Terminal.terminal_width # overflow: hidden;
        accum = 0 and puts if accum + term_width > ANSI::Terminal.terminal_width # overflow: wrap;
        page.driver.resize(width, height)

        page.save_screenshot path
        print ANSI.up(19) + ANSI.right(accum) if accum > 0
        system("bin/imgcat #{path}")
        print ANSI.right(accum) if accum > 0
        puts ANSI.bold { name.center(term_width) }

        if git_status = tracked_screenshots[path]
          passed = false unless git_status[1] == `git hash-object #{path}`.chomp
        end

        accum + term_width + 1
      end
      puts
      return true if passed
    end
  end
end

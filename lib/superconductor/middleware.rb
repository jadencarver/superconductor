require 'superconductor'

module Superconductor
  class Middleware

    PORT = Superconductor.start();
    ENV["GIT_DIR"] = File.join(Dir.pwd, '.git')

    def initialize(app)
      @app = app
      @gem_path = File.expand_path(File.join(File.dirname(__FILE__), '../..'))
      @assets = Dir[File.join(@gem_path, 'assets/**/*.{js,css}')]
      @assets += Dir[File.join(@gem_path, 'spec/integration/screenshots/*.png')]
    end

    def call(env)
      path = env['PATH_INFO']
      serve_asset(path) or serve_xslt(path) or serve_response(env)
    end

    private

    def serve_asset(path)
      if asset = @assets.find { |p| path == p[@gem_path.length..-1] }
        status, headers = 200, {}
        body = [open(asset).read]
        headers['Content-Type'] = content_type(path)
        [status, headers, body]
      end
    end

    def content_type(path)
      case path
      when /.html?$/ then "text/html"
      when /.css$/   then "text/css"
      when /.js$/    then "text/javascript"  
      when /.png$/   then "image/png"
      when /.jpg$/   then "image/jpeg"
      else "text/plain"
      end
    end

    def serve_xslt(path)
      if path == '/__panel.xslt'
        status, headers = 200, {}
        panel_xslt = Superconductor.panel_xslt
        panel_xslt.free = Superconductor[:cleanup]
        body = [panel_xslt.to_s]
        headers['Content-Type'] = 'text/xml'
        [status, headers, body]
      end
    end

    def serve_response(env)
      status, headers, response = *@app.call(env)
      body = []
      response.each do |res|
        body << res
      end
      if (type = headers["Content-Type"]) && type['text/html']
        panel_js = Superconductor.panel_js(PORT)
        panel_js.free = Superconductor[:cleanup]
        headers.delete('Content-Length')
        body << panel_js.to_s
      end
      [status, headers, body]
    end

  end
end

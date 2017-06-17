require 'superconductor'

module Superconductor
  class Middleware

    PORT = Superconductor.start();

    def initialize(app)
      @app = app
      @assets = Dir[File.expand_path('../../../assets/**/*.{js,css}', __FILE__)].map { |d| d[Dir.pwd.length+1..-1] }
      @assets += Dir[File.expand_path('../../../spec/integration/screenshots/*.png', __FILE__)].map { |d| d[Dir.pwd.length+1..-1] }
    end

    def call(env)
      path = env['PATH_INFO']
      serve_asset(path) or serve_xslt(path) or serve_response(env)
    end

    private

    def serve_asset(path)
      if asset = @assets.find { |p| p == path[1..-1] }
        status, headers = 200, {}
        local_path = File.expand_path(File.join('..', '..', '..', asset), __FILE__)
        body = [open(local_path).read]
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
      if headers["Content-Type"] == 'text/html'
        panel_js = Superconductor.panel_js(PORT)
        panel_js.free = Superconductor[:cleanup]
        headers.delete('Content-Length')
        body << panel_js.to_s
      end
      [status, headers, body]
    end

  end
end

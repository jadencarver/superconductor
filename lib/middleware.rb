class Superconductor::Middleware
  def initialize(app)
    @app = app
    @assets = Dir[File.expand_path('../../assets/*', __FILE__)].map { |d| File.basename(d) }
    Superconductor.start();
  end

  def call(env)
    asset = @assets.find { |p| p == env['PATH_INFO'][1..-1] }
    status, headers = 200, {}
    if asset
      body = [open(File.expand_path('../../assets/'+asset, __FILE__)).read]
      headers['Content-Type'] = case File.extname(env['PATH_INFO'])
                                when '.css' then 'text/css'
                                when '.js' then 'text/javascript'
                                when '.png' then 'image/png'
                                when '.jpg', '.jpeg' then 'image/jpeg'
                                end
    elsif env['PATH_INFO'] == '/__panel.xslt'
      panel_xslt = Superconductor.panel_xslt
      panel_xslt.free = Superconductor[:cleanup]
      body = [panel_xslt.to_s]
      headers['Content-Type'] = 'text/xml'
    else
      status, headers, response = *@app.call(env)
      body = []
      response.each do |res|
        body << res
      end
      panel_js = Superconductor.panel_js
      panel_js.free = Superconductor[:cleanup]
      body << panel_js.to_s
    end
    [status, headers, body]
  end
end

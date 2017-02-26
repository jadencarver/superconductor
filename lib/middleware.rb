class Superconductor::Middleware
  def initialize(app)
    @app = app
  end

  def call(env)
    if env['PATH_INFO'] == '/__pm.js'
      status, headers = 200, {}
      body = [open('assets/__pm.js').read]
    else
      status, headers, body = @app.call(env)
      hello = Superconductor.hello_world
      hello.free = Superconductor[:hello_world_free]
    end
    [status, headers, body + [hello.to_s]]
  end
end

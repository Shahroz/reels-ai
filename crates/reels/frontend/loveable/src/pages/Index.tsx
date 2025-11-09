import { Navigation } from '@/components/Navigation';
import { Button } from '@/components/ui/button';
import { Link } from 'react-router-dom';
import { Video, Zap, DollarSign, Sparkles, ArrowRight, CheckCircle } from 'lucide-react';

const Index = () => {
  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      
      {/* Hero Section */}
      <section className="pt-32 pb-20 px-6">
        <div className="container mx-auto max-w-6xl">
          <div className="text-center space-y-8 animate-fade-in-up">
            <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full glass-card">
              <Sparkles className="w-4 h-4 text-accent" />
              <span className="text-sm font-medium">AI-Powered Video Generation</span>
            </div>
            
            <h1 className="text-6xl md:text-7xl font-bold leading-tight">
              Transform Products Into
              <br />
              <span className="text-gradient">Engaging Video Pages</span>
            </h1>
            
            <p className="text-xl text-muted-foreground max-w-2xl mx-auto">
              Simply input product details. Our AI automatically scrapes data, generates stunning video product pages, and helps you convert better.
            </p>
            
            <div className="flex items-center justify-center gap-4 pt-4">
              <Link to="/auth">
                <Button size="lg" className="gradient-accent text-white text-lg px-8 py-6 hover:shadow-lg transition-all">
                  Start Creating Free
                  <ArrowRight className="ml-2 w-5 h-5" />
                </Button>
              </Link>
              <Button size="lg" variant="outline" className="text-lg px-8 py-6">
                Watch Demo
              </Button>
            </div>
          </div>
        </div>
      </section>

      {/* Features Grid */}
      <section className="py-20 px-6 bg-muted/30">
        <div className="container mx-auto max-w-6xl">
          <h2 className="text-4xl font-bold text-center mb-16">
            Everything You Need to
            <span className="text-gradient"> Succeed</span>
          </h2>
          
          <div className="grid md:grid-cols-3 gap-8">
            <FeatureCard
              icon={<Zap className="w-8 h-8" />}
              title="AI-Powered Automation"
              description="Automatically scrape product data, specs, reviews, and visuals from any source."
            />
            <FeatureCard
              icon={<Video className="w-8 h-8" />}
              title="Video Generation"
              description="Transform static product info into dynamic, engaging video pages that convert."
            />
            <FeatureCard
              icon={<DollarSign className="w-8 h-8" />}
              title="Credit System"
              description="Pay-as-you-go with credits. Start with 10 free credits, buy more anytime."
            />
          </div>
        </div>
      </section>

      {/* How It Works */}
      <section className="py-20 px-6">
        <div className="container mx-auto max-w-6xl">
          <h2 className="text-4xl font-bold text-center mb-16">
            Simple 4-Step Process
          </h2>
          
          <div className="grid md:grid-cols-4 gap-6">
            <StepCard number="1" title="Input Details" description="Enter product title, description, images, and price" />
            <StepCard number="2" title="AI Scrapes Data" description="System automatically finds specs, reviews, and more" />
            <StepCard number="3" title="Video Generated" description="AI creates your stunning video product page" />
            <StepCard number="4" title="Share & Embed" description="Get shareable links and embed codes instantly" />
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 px-6 gradient-hero">
        <div className="container mx-auto max-w-4xl text-center text-white space-y-6">
          <h2 className="text-5xl font-bold">Ready to Transform Your Products?</h2>
          <p className="text-xl opacity-90">Start with 10 free credits. No credit card required.</p>
          <Link to="/auth">
            <Button size="lg" className="bg-white text-primary hover:bg-white/90 text-lg px-8 py-6 mt-6">
              Get Started Now
              <ArrowRight className="ml-2 w-5 h-5" />
            </Button>
          </Link>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-12 px-6 border-t">
        <div className="container mx-auto text-center text-muted-foreground">
          <p>&copy; 2025 VideoGen. All rights reserved.</p>
        </div>
      </footer>
    </div>
  );
};

const FeatureCard = ({ icon, title, description }: { icon: React.ReactNode; title: string; description: string }) => (
  <div className="glass-card p-8 rounded-2xl hover:shadow-lg transition-all group">
    <div className="w-16 h-16 rounded-xl gradient-accent flex items-center justify-center mb-6 text-white group-hover:scale-110 transition-transform">
      {icon}
    </div>
    <h3 className="text-2xl font-bold mb-3">{title}</h3>
    <p className="text-muted-foreground leading-relaxed">{description}</p>
  </div>
);

const StepCard = ({ number, title, description }: { number: string; title: string; description: string }) => (
  <div className="text-center space-y-4">
    <div className="w-16 h-16 rounded-full gradient-accent flex items-center justify-center text-white text-2xl font-bold mx-auto">
      {number}
    </div>
    <h3 className="text-xl font-bold">{title}</h3>
    <p className="text-muted-foreground text-sm">{description}</p>
  </div>
);

export default Index;
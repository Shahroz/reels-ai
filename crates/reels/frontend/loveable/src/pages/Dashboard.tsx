import { useState, useEffect } from 'react';
import { Navigation } from '@/components/Navigation';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Plus, Video, CreditCard, TrendingUp } from 'lucide-react';
import { useAuth } from '@/hooks/useAuth';
import { useNavigate } from 'react-router-dom';
import { supabase } from '@/integrations/supabase/client';
import ProductForm from '@/components/ProductForm';
import VideoPagesList from '@/components/VideoPagesList';

const Dashboard = () => {
  const { user, loading } = useAuth();
  const navigate = useNavigate();
  const [credits, setCredits] = useState(0);
  const [showProductForm, setShowProductForm] = useState(false);
  const [stats, setStats] = useState({
    totalVideos: 0,
    totalViews: 0,
  });

  useEffect(() => {
    if (!loading && !user) {
      navigate('/auth');
    }
  }, [user, loading, navigate]);

  useEffect(() => {
    if (user) {
      loadCredits();
      loadStats();
    }
  }, [user]);

  const loadCredits = async () => {
    if (!user) return;
    
    const { data } = await supabase
      .from('credits')
      .select('balance')
      .eq('user_id', user.id)
      .single();
    
    if (data) {
      setCredits(data.balance);
    }
  };

  const loadStats = async () => {
    if (!user) return;
    
    const { data: videos } = await supabase
      .from('video_pages')
      .select('views')
      .eq('user_id', user.id);
    
    if (videos) {
      setStats({
        totalVideos: videos.length,
        totalViews: videos.reduce((sum, v) => sum + v.views, 0),
      });
    }
  };

  if (loading) {
    return <div className="min-h-screen bg-background" />;
  }

  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      
      <div className="pt-24 px-6 pb-12">
        <div className="container mx-auto max-w-7xl">
          {/* Header */}
          <div className="mb-8">
            <h1 className="text-4xl font-bold mb-2">Dashboard</h1>
            <p className="text-muted-foreground">Manage your video product pages</p>
          </div>

          {/* Stats Grid */}
          <div className="grid md:grid-cols-3 gap-6 mb-8">
            <StatsCard
              icon={<CreditCard className="w-6 h-6" />}
              title="Available Credits"
              value={credits.toString()}
              gradient="gradient-accent"
            />
            <StatsCard
              icon={<Video className="w-6 h-6" />}
              title="Total Videos"
              value={stats.totalVideos.toString()}
              gradient="gradient-primary"
            />
            <StatsCard
              icon={<TrendingUp className="w-6 h-6" />}
              title="Total Views"
              value={stats.totalViews.toString()}
              gradient="gradient-accent"
            />
          </div>

          {/* Action Button */}
          {!showProductForm && (
            <div className="mb-8">
              <Button
                size="lg"
                className="gradient-accent text-white"
                onClick={() => setShowProductForm(true)}
              >
                <Plus className="mr-2" />
                Create New Video Page
              </Button>
            </div>
          )}

          {/* Product Form */}
          {showProductForm && (
            <div className="mb-8">
              <ProductForm
                onClose={() => {
                  setShowProductForm(false);
                  loadCredits();
                  loadStats();
                }}
              />
            </div>
          )}

          {/* Video Pages List */}
          <VideoPagesList />
        </div>
      </div>
    </div>
  );
};

const StatsCard = ({ icon, title, value, gradient }: { icon: React.ReactNode; title: string; value: string; gradient: string }) => (
  <Card className="glass-card">
    <CardHeader className="flex flex-row items-center justify-between pb-2">
      <CardTitle className="text-sm font-medium text-muted-foreground">{title}</CardTitle>
      <div className={`w-10 h-10 rounded-lg ${gradient} flex items-center justify-center text-white`}>
        {icon}
      </div>
    </CardHeader>
    <CardContent>
      <div className="text-3xl font-bold">{value}</div>
    </CardContent>
  </Card>
);

export default Dashboard;
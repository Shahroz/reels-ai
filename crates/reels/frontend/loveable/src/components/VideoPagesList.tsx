import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ExternalLink, Eye, Copy } from 'lucide-react';
import { useAuth } from '@/hooks/useAuth';
import { supabase } from '@/integrations/supabase/client';
import { useToast } from '@/components/ui/use-toast';

interface VideoPage {
  id: string;
  product_id: string;
  status: string;
  share_url: string;
  views: number;
  created_at: string;
  products: {
    title: string;
    description: string;
    price: number;
  };
}

const VideoPagesList = () => {
  const { user } = useAuth();
  const { toast } = useToast();
  const [videoPages, setVideoPages] = useState<VideoPage[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (user) {
      loadVideoPages();
    }
  }, [user]);

  const loadVideoPages = async () => {
    if (!user) return;

    const { data, error } = await supabase
      .from('video_pages')
      .select(`
        id,
        product_id,
        status,
        share_url,
        views,
        created_at,
        products (
          title,
          description,
          price
        )
      `)
      .eq('user_id', user.id)
      .order('created_at', { ascending: false });

    if (error) {
      console.error('Error loading video pages:', error);
    } else {
      setVideoPages(data as any);
    }

    setLoading(false);
  };

  const copyToClipboard = (url: string) => {
    navigator.clipboard.writeText(url);
    toast({
      title: 'Copied!',
      description: 'Share URL copied to clipboard',
    });
  };

  if (loading) {
    return <div className="text-center py-12 text-muted-foreground">Loading...</div>;
  }

  if (videoPages.length === 0) {
    return (
      <Card className="glass-card">
        <CardContent className="py-12 text-center">
          <p className="text-muted-foreground">No video pages yet. Create your first one!</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold mb-4">Your Video Pages</h2>
      
      {videoPages.map((page) => (
        <Card key={page.id} className="glass-card hover:shadow-lg transition-all">
          <CardHeader>
            <div className="flex items-start justify-between">
              <div>
                <CardTitle className="text-xl">{page.products.title}</CardTitle>
                <CardDescription className="mt-2">
                  {page.products.description || 'No description'}
                </CardDescription>
              </div>
              <Badge
                variant={page.status === 'completed' ? 'default' : 'secondary'}
                className={page.status === 'completed' ? 'gradient-accent text-white' : ''}
              >
                {page.status}
              </Badge>
            </div>
          </CardHeader>
          <CardContent>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-4 text-sm text-muted-foreground">
                <div className="flex items-center gap-1">
                  <Eye className="w-4 h-4" />
                  {page.views} views
                </div>
                <div>${page.products.price}</div>
              </div>
              
              <div className="flex items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => copyToClipboard(page.share_url)}
                >
                  <Copy className="w-4 h-4 mr-2" />
                  Copy Link
                </Button>
                <Button
                  variant="default"
                  size="sm"
                  className="gradient-accent text-white"
                  onClick={() => window.open(page.share_url, '_blank')}
                >
                  <ExternalLink className="w-4 h-4 mr-2" />
                  View
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
};

export default VideoPagesList;